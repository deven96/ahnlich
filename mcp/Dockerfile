FROM python:3.11-slim-bookworm AS builder

WORKDIR /app

ENV UV_COMPILE_BYTECODE=1 \
    UV_LINK_MODE=copy

RUN pip install --no-cache-dir uv==0.11.31

COPY pyproject.toml uv.lock README.md LICENSE ./
COPY src ./src

RUN uv sync --locked --no-dev --no-editable


FROM python:3.11-slim-bookworm AS runtime

RUN useradd \
    --create-home \
    --uid 10001 \
    --shell /usr/sbin/nologin \
    ahnlich

WORKDIR /app

ENV PATH="/app/.venv/bin:$PATH" \
    PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1

COPY --from=builder /app/.venv /app/.venv

USER ahnlich

ENTRYPOINT ["ahnlich-mcp"]
CMD ["--profile", "ai"]