# AGENTS.md - IoT Services Codebase

## Overview
Rust IoT project with microservices via Kafka and MQTT:
- **device_service**: Rust - MQTT consumer, produces to Kafka topic-a
- **control_service**: Rust - consumes from topic-a, produces to topic-b
- **notification-service**: Python - consumes service-notifications, sends emails
- **temperature_sensor**: Python - publishes to MQTT

## Architecture
```
MQTT (1883) --> device_service --> Kafka (topic-a) --> control_service --> Kafka (topic-b) --> notification-service
```

## Build & Run

### Rust
```bash
cargo build --release
cargo build -p device_service --release
cargo run -p device_service
cargo run -p control_service
```

### Docker
```bash
docker-compose up --build
docker-compose up -d
docker-compose logs -f device_service
docker-compose down
```

### Python
```bash
# notification-service
cd notification-service && pip install -r requirements.txt && python main.py

# temperature_sensor
cd temperature_sensor && pip install -r requirements.txt && python simulator.py
```

## Lint, Format & Type Check

### Rust
```bash
cargo clippy --all-targets -- -D warnings  # fails on warnings
cargo clippy -p device_service -- -D warnings
cargo fmt && cargo fmt --check
cargo check
# All: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo check
```

### Python
```bash
pip install pylint
pylint notification-service/main.py
pylint temperature_sensor/*.py
```

## Testing

### Rust
```bash
cargo test                    # all tests
cargo test -p device_service # specific crate
cargo test test_name         # single test by name
cargo test -- --nocapture    # verbose with prints
```

### Python
```bash
pytest -v
pytest -k test_name
```

## Code Style

### Rust Naming
- Types: PascalCase (`MessagePayload`)
- Functions/variables: snake_case
- Constants: SCREAMING_SNAKE_CASE
- Crates: snake_case (`device_service`)

### Rust Imports (order)
1. Standard library (`std::`, `core::`)
2. External crates (`tokio::`, `serde::`, `rdkafka::`, `rumqttc::`)
3. First-party (local modules)

```rust
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::message::Message;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use rumqttc::{Event, MqttOptions, AsyncClient, Packet};
```

### Rust Error Handling
- `.expect()` for unrecoverable errors (client creation)
- `match` with `eprintln!()` for recoverable errors

```rust
let producer: FutureProducer = ClientConfig::new()
    .set("bootstrap.servers", "kafka:9092")
    .create()
    .expect("Failed to create producer");
```

### Rust Async
- Use `#[tokio::main]` for async main
- Use `tokio::select!` for concurrent tasks
- Clone producers/consumers before passing to spawned tasks
- Use `tokio::spawn` for background tasks

### Rust Types
- Annotate public function return types explicitly
- Use explicit Kafka client types (`FutureProducer`, `StreamConsumer`)
- Derive `Debug, Clone, Serialize, Deserialize` for data structs
- Use `serde_json::to_string()` / `from_slice()` for JSON

### Python Code Style
- Follow PEP 8, use type hints, use logging

```python
import logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)
```

## Kafka Topics
- `topic-a`: Messages from device_service
- `topic-b`: Messages from control_service
- `service-notifications`: Startup/shutdown events

## MQTT Topics
- `temperature/data`: Temperature sensor readings (JSON)

## Project Structure
```
.
├── device_service/     # Rust - MQTT to Kafka
├── control_service/    # Rust - Kafka consumer/producer
├── notification-service/ # Python - Email alerts
├── temperature_sensor/ # Python - Sensor simulator
├── mosquitto/config/   # MQTT broker config
├── kafka/              # Kafka broker setup
├── docker-compose.yml
└── AGENTS.md
```

## Key Files
- `device_service/src/main.rs:20-26`: Producer, MQTT setup
- `device_service/src/main.rs:50-73`: MQTT to Kafka
- `control_service/src/main.rs:16-30`: Producer/consumer
- `control_service/src/main.rs:36-66`: Background producer
- `notification-service/main.py`: Email sending

## Common Tasks

### Add Rust dependency
1. Edit service's Cargo.toml
2. Run `cargo update`

### Add Kafka topic
1. Update consumer `subscribe()` with topic name
2. Update `FutureRecord::to("topic-name")`

### Add MQTT subscription
```rust
client.subscribe("topic/name", rumqttc::QoS::AtMostOnce).await.expect("Failed to subscribe");
```

## Environment
- Kafka: `kafka:9092` (Docker) / `localhost:9092` (local)
- MQTT: `mqtt-broker:1883` (Docker) / `localhost:1883` (local)
