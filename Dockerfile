# syntax=docker/dockerfile:1

# ---- Build stage ----
FROM rust:1-slim-bookworm AS builder
# native-tls -> openssl-sys needs pkg-config + libssl-dev at build time.
RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Cache dependencies: build a stub against the real manifests first.
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src \
    && echo 'fn main() {}' > src/main.rs \
    && echo '' > src/lib.rs \
    && cargo build --release --bin pp-ss-r 2>/dev/null || true \
    && rm -rf src

# Build the real binary.
COPY src ./src
COPY data ./data
RUN touch src/main.rs src/lib.rs && cargo build --release --bin pp-ss-r

# ---- Runtime stage ----
FROM debian:bookworm-slim AS runtime
# TLS at runtime: libssl3 for native-tls, ca-certificates to verify the API's cert.
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*
RUN useradd --system --uid 10001 appuser
WORKDIR /app
COPY --from=builder /app/target/release/pp-ss-r /usr/local/bin/pp-ss-r
# The payload (data/order.json) is baked into the binary at compile time;
# only the runtime-served static assets need to be present.
COPY static ./static

ENV BIND_ADDR=0.0.0.0:8080 \
    DATA_BASE_URL=http://127.0.0.1:8080 \
    RUST_LOG=info,actix_server::worker=warn
EXPOSE 8080
USER appuser
CMD ["pp-ss-r"]
