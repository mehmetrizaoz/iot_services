# IoT Sensor Management System

A distributed IoT sensor management platform that collects, processes, and visualizes sensor data from multiple sensor types (temperature, velocity, position) using MQTT, Kafka, and a React frontend.

## Architecture

```
MQTT (1883) --> device_service --> Kafka (topic-a) --> control_service --> Kafka (topic-b)
                                                                           |
UI (React) <-- HTTP polling -- nginx (8080) <------- control_service <-----+
                         |
                         +--> notification-service --> Email
```

## Technologies

- **Frontend**: React, JavaScript
- **Backend Services**: Rust (device_service, control_service, sensor_storage)
- **Message Brokers**: Apache Kafka, Eclipse Mosquitto (MQTT)
- **Database**: PostgreSQL
- **Reverse Proxy**: nginx
- **Containerization**: Docker, Docker Compose

## Quick Start

```bash
# Build and start all services
docker-compose up --build

# Start in detached mode
docker-compose up -d

# View logs
docker-compose logs -f

# Stop all services
docker-compose down
```

## Access Points

| Service | URL |
|---------|-----|
| React UI | http://localhost:8080 |
| API (via nginx) | http://localhost:8080/api/ |

## Services

| Service | Description | Port |
|---------|-------------|------|
| mqtt-broker | Eclipse Mosquitto MQTT broker | 1883 |
| kafka | Apache Kafka message broker | 9092 |
| postgres | PostgreSQL database | 5432 |
| device_service | Rust MQTT consumer, Kafka producer | 8081 |
| control_service | Kafka consumer, MQTT producer, HTTP API | 8080 |
| notification-service | Email notification service | - |
| user_interface | React frontend | 3000 |
| nginx | Reverse proxy | 80/8080 |

## Kafka Topics

- `topic-a`: Sensor data from device_service
- `topic-b`: Control commands from control_service
- `service-notifications`: Startup/shutdown events

## MQTT Topics

- `temperature/data`: Temperature sensor readings
- `velocity/data`: Velocity sensor readings
- `position/data`: Position sensor readings
- `sensors/settings/{type}`: Sensor configuration commands

## Development

### Rust Services

```bash
cargo build --release
cargo run -p device_service
cargo run -p control_service
cargo test
cargo clippy --all-targets -- -D warnings
```

### Python Services

```bash
cd notification-service && pip install -r requirements.txt && python main.py
cd temperature_sensor && pip install -r requirements.txt && python simulator.py
```

## License

MIT License - see [LICENSE](LICENSE) file
