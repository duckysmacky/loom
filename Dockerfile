FROM rust:1-slim-trixie AS builder
WORKDIR /build

COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src

ENV SQLX_OFFLINE=true
RUN cargo build --release

FROM debian:trixie-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --shell /usr/sbin/nologin loom

COPY --from=builder /build/target/release/loom /usr/local/bin/loom

USER loom
EXPOSE 8080
ENTRYPOINT ["loom"]
