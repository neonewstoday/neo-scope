FROM rust:1.97.1-slim-bookworm AS builder
WORKDIR /app
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked --package neo-scope-indexer --package neo-scope-api \
    && cp target/release/neo-scope-indexer target/release/neo-scope-api /usr/local/bin/

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 neoscope
COPY --from=builder /usr/local/bin/neo-scope-indexer /usr/local/bin/neo-scope-api /usr/local/bin/
USER neoscope
EXPOSE 8083
CMD ["neo-scope-api"]
