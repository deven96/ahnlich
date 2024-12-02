FROM lukemathwalker/cargo-chef:latest-rust-1.81.0 AS chef
WORKDIR /app
RUN apt update && apt install lld clang -y
FROM chef AS planner
COPY . /app/


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

# Build our project
RUN cargo build --release --bin ${AHNLICH_BIN}
RUN cargo build --release --bin ahnlich-cli


FROM debian:bookworm-slim AS runtime

ARG AHNLICH_BIN
ARG DEFAULT_PORT
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends openssl ca-certificates \
    libsasl2-modules-gssapi-mit libsasl2-dev \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*


COPY --from=builder /app/target/release/${AHNLICH_BIN} /usr/local/bin/
COPY --from=builder /app/target/release/ahnlich-cli /usr/local/bin/

ENV AHNLICH_BIN=${AHNLICH_BIN}


ENTRYPOINT ["sh", "-c"]

EXPOSE ${DEFAULT_PORT}

CMD ["$AHNLICH_BIN"]
