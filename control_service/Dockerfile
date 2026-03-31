FROM rust:1.87 as builder

WORKDIR /app
RUN apt-get update && apt-get install -y cmake libssl-dev pkg-config

COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

COPY src ./src
RUN rm -rf target/release/deps/control_service* && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/control_service /usr/local/bin/

CMD ["control_service"]
