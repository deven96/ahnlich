ARG RUST_VERSION
ARG AHNLICH_BIN=ahnlich-db
ARG CUDA_RUNTIME_IMAGE=nvidia/cuda:12.8.1-cudnn-runtime-ubuntu24.04@sha256:ac55d124da4882b497f732d8dfd9a702d5447a5f29d08d56da6f64f0a1eb34bc
FROM lukemathwalker/cargo-chef:latest-rust-${RUST_VERSION} AS chef
WORKDIR /app
RUN apt update && apt install lld clang protobuf-compiler -y 
FROM chef AS planner
COPY . /app/

# remove types build.rs
RUN rm types/build.rs

# Compute a lock-like file for our project
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
ARG AHNLICH_BIN
ARG DEFAULT_PORT=1369
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json
# Up to this point, if our dependency tree stays the same,
# all layers should be cached.
COPY . /app/

# remove types build.rs
RUN rm types/build.rs
# Build our project
RUN cargo build --release --bin ${AHNLICH_BIN}
RUN cargo build --release --bin ahnlich-cli

# ort-sys exposes the provider libraries as symlinks into its build cache on
# Linux. Resolve them here so the final image gets the matching library files.
RUN if [ "$AHNLICH_BIN" = "ahnlich-ai" ]; then \
      install -Dm644 -t /app/ort-cuda \
        target/release/libonnxruntime_providers_shared.so \
        target/release/libonnxruntime_providers_cuda.so; \
    fi

FROM ${CUDA_RUNTIME_IMAGE} AS nvidia-runtime
# Copy CUDA user-space libraries only. The container runtime supplies the GPU
# driver from the host. Include cuDNN's component libraries and JIT dependencies.
RUN set -eu; \
    mkdir -p /cuda-libs; \
    ldconfig; \
    ldconfig -p | awk '$1 ~ /^lib(cublas(Lt)?|cudart|cudnn[[:alnum:]_]*|cufft|curand|nvJitLink|nvrtc(-builtins)?)\.so(\.|$)/ {print $1, $NF}' \
      | while read -r soname path; do cp -L "$path" "/cuda-libs/$soname"; done; \
    test -f /cuda-libs/libcudnn.so.9

FROM debian:trixie-slim AS runtime-base
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends openssl ca-certificates \
  libsasl2-modules-gssapi-mit libsasl2-dev curl \
  # Clean up
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*

FROM runtime-base AS ahnlich-db-runtime

FROM runtime-base AS ahnlich-ai-runtime
COPY --from=nvidia-runtime /cuda-libs/ /opt/ahnlich/cuda/lib/
COPY --from=nvidia-runtime /NGC-DL-CONTAINER-LICENSE /usr/share/doc/ahnlich-cuda/
COPY --from=nvidia-runtime /usr/share/doc/libcudnn9-cuda-12/copyright /usr/share/doc/ahnlich-cuda/cudnn-copyright
# The statically linked runtime loads its providers by library name.
COPY --from=builder /app/ort-cuda/ /opt/ahnlich/cuda/lib/
RUN printf '%s\n' /opt/ahnlich/cuda/lib > /etc/ld.so.conf.d/ahnlich-cuda.conf \
  && ldconfig \
  && ldconfig -p | grep -F 'libonnxruntime_providers_shared.so' \
  && ldd /opt/ahnlich/cuda/lib/libonnxruntime_providers_cuda.so > /tmp/ort-cuda-ldd \
  && if grep -F 'not found' /tmp/ort-cuda-ldd; then exit 1; fi \
  && rm /tmp/ort-cuda-ldd

FROM ${AHNLICH_BIN}-runtime AS runtime
ARG AHNLICH_BIN
ARG DEFAULT_PORT

COPY --from=builder /app/target/release/${AHNLICH_BIN} /usr/local/bin/
COPY --from=builder /app/target/release/ahnlich-cli /usr/local/bin/

ENV AHNLICH_BIN=${AHNLICH_BIN}


ENTRYPOINT ["sh", "-c"]

EXPOSE ${DEFAULT_PORT}

CMD ["$AHNLICH_BIN"]
