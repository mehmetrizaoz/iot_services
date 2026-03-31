use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use futures_util::StreamExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SensorData {
    device_id: String,
    sensor_type: String,
    value: serde_json::Value,
    unit: String,
    timestamp: u64,
}

async fn init_database(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sensor_readings (
            id SERIAL PRIMARY KEY,
            device_id VARCHAR(100) NOT NULL,
            sensor_type VARCHAR(50) NOT NULL,
            value JSONB NOT NULL,
            unit VARCHAR(20) NOT NULL,
            timestamp BIGINT NOT NULL,
            recorded_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_sensor_readings_device_id ON sensor_readings(device_id)"
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_sensor_readings_timestamp ON sensor_readings(timestamp)"
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_sensor_readings_type ON sensor_readings(sensor_type)"
    )
    .execute(pool)
    .await?;

    println!("Database initialized successfully");
    Ok(())
}

async fn save_reading(pool: &PgPool, data: &SensorData) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO sensor_readings (device_id, sensor_type, value, unit, timestamp) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(&data.device_id)
    .bind(&data.sensor_type)
    .bind(serde_json::Value::from(data.value.clone()))
    .bind(&data.unit)
    .bind(data.timestamp as i64)
    .execute(pool)
    .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Sensor Storage Service starting...");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@postgres:5432/sensordb".to_string());

    eprintln!("Connecting to PostgreSQL...");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    eprintln!("Connected to PostgreSQL");
    init_database(&pool).await?;

    eprintln!("Connecting to Kafka...");
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", "kafka:9092")
        .set("group.id", "sensor-storage-consumer-group")
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "earliest")
        .create()?;

    consumer.subscribe(&["topic-a"])?;
    eprintln!("Subscribed to topic-a");

    eprintln!("Sensor Storage Service started - listening for sensor data");

    let consumer_stream = consumer.stream();

    consumer_stream
        .for_each_concurrent(10, |message| {
            let pool = pool.clone();
            async move {
                match message {
                    Ok(msg) => {
                        if let Some(payload) = msg.payload() {
                            match serde_json::from_slice::<SensorData>(payload) {
                                Ok(data) => {
                                    eprintln!("RECEIVED: device_id={}, type={}, value={:?} {}, timestamp={}", 
                                        data.device_id, data.sensor_type, data.value, data.unit, data.timestamp);
                                    match save_reading(&pool, &data).await {
                                        Ok(_) => {
                                            eprintln!("SAVED to database: device_id={}, type={}", data.device_id, data.sensor_type);
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to save reading: {:?}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to parse message: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Kafka error: {:?}", e);
                    }
                }
            }
        })
        .await;

    Ok(())
}
