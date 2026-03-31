use bytes::Bytes;
use tokio_stream::StreamExt;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use parking_lot::RwLock;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::Message;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SensorSettings {
    temperature: SensorConfig,
    velocity: SensorConfig,
    position: PositionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SensorConfig {
    min: f64,
    max: f64,
    interval: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PositionConfig {
    x_range: f64,
    y_range: f64,
    z_range: f64,
    interval: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ControlCommand {
    command_id: u32,
    sensor_type: String,
    settings: serde_json::Value,
    timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SensorData {
    device_id: String,
    sensor_type: String,
    value: serde_json::Value,
    unit: String,
    timestamp: u64,
}

#[derive(Clone)]
struct LatestData {
    temperature: Option<SensorData>,
    velocity: Option<SensorData>,
    position: Option<SensorData>,
}

#[derive(Clone)]
struct AppState {
    settings_tx: mpsc::Sender<SensorSettings>,
    latest_data: Arc<RwLock<LatestData>>,
}

async fn handle_request(
    req: Request<Incoming>,
    state: AppState,
) -> Result<Response<Full<Bytes>>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/api/settings") | (&Method::POST, "/api/settings/") => {
            let body_bytes = req.collect().await.unwrap().to_bytes();
            match serde_json::from_slice::<SensorSettings>(&body_bytes) {
                Ok(settings) => {
                    println!("\n[SETTINGS RECEIVED]");
                    println!(
                        "  Temperature: {}°C - {}°C, every {}s",
                        settings.temperature.min, settings.temperature.max, settings.temperature.interval
                    );
                    println!(
                        "  Velocity: {} - {} rpm, every {}s",
                        settings.velocity.min, settings.velocity.max, settings.velocity.interval
                    );
                    println!(
                        "  Position: x±{} y±{} z±{}m, every {}s\n",
                        settings.position.x_range,
                        settings.position.y_range,
                        settings.position.z_range,
                        settings.position.interval
                    );

                    let _ = state.settings_tx.send(settings).await;

                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", "application/json")
                        .body(Full::new(Bytes::from(r#"{"status":"ok"}"#)))
                        .unwrap())
                }
                Err(e) => Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Full::new(Bytes::from(format!("Invalid JSON: {}", e))))
                    .unwrap()),
            }
        }
        (&Method::GET, "/api/data") | (&Method::GET, "/api/data/") => {
            let latest = state.latest_data.read().clone();
            let response = serde_json::json!({
                "temperature": latest.temperature,
                "velocity": latest.velocity,
                "position": latest.position
            });
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(Full::new(Bytes::from(serde_json::to_string(&response).unwrap())))
                .unwrap())
        }
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::new(Bytes::from("Not Found")))
            .unwrap()),
    }
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<SensorSettings>(10);
    let latest_data = Arc::new(RwLock::new(LatestData {
        temperature: None,
        velocity: None,
        position: None,
    }));

    let state = AppState { 
        settings_tx: tx, 
        latest_data: latest_data.clone() 
    };

    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", "kafka:9092")
        .set("group.id", "control-service-consumer-group")
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("Failed to create consumer");

    consumer.subscribe(&["topic-a"]).expect("Failed to subscribe");

    println!("Control Service started - HTTP + MQTT");

    let http_state = state.clone();

    let http_handle = tokio::spawn(async move {
        let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
        println!("HTTP server starting on 0.0.0.0:8080");

        loop {
            if let Ok((stream, _)) = listener.accept().await {
                let http_state = http_state.clone();

                tokio::spawn(async move {
                    let io = TokioIo::new(stream);
                    let service = service_fn(move |req| {
                        let http_state = http_state.clone();
                        async move { handle_request(req, http_state).await }
                    });

                    if let Err(e) = http1::Builder::new()
                        .serve_connection(io, service)
                        .await
                    {
                        eprintln!("Connection error: {:?}", e);
                    }
                });
            }
        }
    });

    let latest_data_for_kafka = latest_data.clone();
    let kafka_handle = tokio::spawn(async move {
        let mut stream = consumer.stream();
        loop {
            match stream.next().await {
                Some(Ok(msg)) => {
                    if let Some(payload) = msg.payload() {
                        if let Ok(sensor) = serde_json::from_slice::<SensorData>(payload) {
                            println!(
                                "[RECEIVED] {}: {} {:?}",
                                sensor.sensor_type, sensor.device_id, sensor.value
                            );

                            let mut data = latest_data_for_kafka.write();
                            match sensor.sensor_type.as_str() {
                                "temperature" => data.temperature = Some(sensor),
                                "velocity" => data.velocity = Some(sensor),
                                "position" => data.position = Some(sensor),
                                _ => {}
                            }
                        }
                    }
                }
                Some(Err(e)) => {
                    eprintln!("[ERROR] Kafka: {:?}", e);
                }
                None => {
                    eprintln!("[ERROR] Kafka stream ended");
                    break;
                }
            }
        }
    });

    let mqtt_handle = tokio::spawn(async move {
        let mut mqtt_options = rumqttc::MqttOptions::new("control_service", "mqtt-broker", 1883);
        mqtt_options.set_keep_alive(Duration::from_secs(30));
        let (client, mut eventloop) = rumqttc::AsyncClient::new(mqtt_options, 100);

        loop {
            tokio::select! {
                notification = eventloop.poll() => {
                    match notification {
                        Ok(rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_))) => {
                            println!("MQTT connected");
                        }
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("MQTT error: {:?}", e);
                        }
                    }
                }
                Some(settings) = rx.recv() => {
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();

                    let commands = vec![
                        ("temperature", 1, serde_json::json!({"min": settings.temperature.min, "max": settings.temperature.max, "interval": settings.temperature.interval})),
                        ("velocity", 2, serde_json::json!({"min": settings.velocity.min, "max": settings.velocity.max, "interval": settings.velocity.interval})),
                        ("position", 3, serde_json::json!({"x_range": settings.position.x_range, "y_range": settings.position.y_range, "z_range": settings.position.z_range, "interval": settings.position.interval})),
                    ];

                    for (sensor_type, cmd_id, settings_val) in commands {
                        let cmd = ControlCommand {
                            command_id: cmd_id,
                            sensor_type: sensor_type.to_string(),
                            settings: settings_val,
                            timestamp,
                        };

                        if let Ok(payload) = serde_json::to_string(&cmd) {
                            let topic = format!("sensors/settings/{}", sensor_type);
                            match client.publish(&topic, rumqttc::QoS::AtMostOnce, false, payload.as_bytes()).await {
                                Ok(_) => println!("[SENT] {} settings to {}", sensor_type, topic),
                                Err(e) => eprintln!("[ERROR] Failed to publish {}: {:?}", sensor_type, e),
                            }
                        }
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = http_handle => {}
        _ = kafka_handle => {}
        _ = mqtt_handle => {}
    }
}
