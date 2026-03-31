FROM rust:1.88 as builder

RUN apt-get update && apt-get install -y cmake libssl-dev pkg-config

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src

RUN cargo build --release
RUN strip target/release/sensor_storage

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl3 ca-certificates libpq-dev && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/sensor_storage /usr/local/bin/sensor_storage

CMD ["sensor_storage"]
