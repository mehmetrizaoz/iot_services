use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::message::Message;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use rumqttc::{Event, MqttOptions, AsyncClient, Packet};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SensorData {
    device_id: String,
    sensor_type: String,
    value: serde_json::Value,
    unit: String,
    timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServiceStartupMessage {
    service_name: String,
    status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ControlCommand {
    command_id: u32,
    random_value: u32,
    timestamp: u64,
}

#[tokio::main]
async fn main() {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "kafka:9092")
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Failed to create producer");

    let kafka_consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", "kafka:9092")
        .set("group.id", "device-service-kafka-consumer")
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("Failed to create kafka consumer");

    kafka_consumer.subscribe(&["topic-c"]).expect("Failed to subscribe to topic-c");

    println!("Device Service started - MQTT to Kafka bridge");

    let startup_msg = ServiceStartupMessage {
        service_name: "device_service".to_string(),
        status: "started".to_string(),
    };
    let payload = serde_json::to_string(&startup_msg).unwrap();
    let record = FutureRecord::to("service-notifications")
        .payload(&payload)
        .key("device_service");
    let _ = producer.send(record, Duration::from_secs(5)).await;

    let mqtt_options = MqttOptions::new("device_service", "mqtt-broker", 1883);
    let (mqtt_client, mut eventloop) = AsyncClient::new(mqtt_options, 100);
    
    mqtt_client.subscribe("sensors/temperature", rumqttc::QoS::AtMostOnce).await.expect("Failed to subscribe");
    mqtt_client.subscribe("sensors/velocity", rumqttc::QoS::AtMostOnce).await.expect("Failed to subscribe");
    mqtt_client.subscribe("sensors/position", rumqttc::QoS::AtMostOnce).await.expect("Failed to subscribe");

    println!("Subscribed to MQTT: sensors/temperature, sensors/velocity, sensors/position");
    println!("Subscribed to Kafka: topic-c");

    let producer_for_mqtt = producer.clone();
    let mqtt_client_clone = mqtt_client.clone();

    tokio::spawn(async move {
        loop {
            match kafka_consumer.recv().await {
                Ok(message) => {
                    if let Some(payload) = message.payload() {
                        if let Ok(cmd) = serde_json::from_slice::<ControlCommand>(payload) {
                            println!("[RECEIVED] Control command from Kafka topic-c: command_id={}, random_value={}, timestamp={}", 
                                cmd.command_id, cmd.random_value, cmd.timestamp);
                            
                            let mqtt_payload = serde_json::to_vec(&cmd).unwrap();
                            match mqtt_client_clone.publish("control/command", rumqttc::QoS::AtMostOnce, false, mqtt_payload).await {
                                Ok(_) => {
                                    println!("[FORWARDED] Control command #{} -> MQTT: control/command (random_value={})", 
                                        cmd.command_id, cmd.random_value);
                                }
                                Err(e) => {
                                    eprintln!("[ERROR] Failed to publish to MQTT: {:?}", e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Kafka error: {:?}", e);
                }
            }
        }
    });

    let producer_for_kafka = producer_for_mqtt.clone();
    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(event) => {
                    if let Event::Incoming(Packet::Publish(publish)) = event {
                        let topic = publish.topic.clone();
                        
                        if let Ok(sensor_data) = serde_json::from_slice::<SensorData>(&publish.payload) {
                            println!("[RECEIVED] Sensor data from MQTT '{}': device_id={}, type={}, value={:?} {}, timestamp={}", 
                                topic,
                                sensor_data.device_id, 
                                sensor_data.sensor_type, 
                                sensor_data.value,
                                sensor_data.unit,
                                sensor_data.timestamp);
                            
                            let payload: Vec<u8> = publish.payload.to_vec();
                            let record = FutureRecord::to("topic-a")
                                .payload(&payload)
                                .key(&sensor_data.device_id);
                            
                            match producer_for_kafka.send(record, Duration::from_secs(5)).await {
                                Ok((partition, offset)) => {
                                    println!("[FORWARDED] {} -> Kafka topic-a: partition={}, offset={}", 
                                        sensor_data.sensor_type, partition, offset);
                                }
                                Err((err, _)) => {
                                    eprintln!("[ERROR] Failed to send to Kafka: {:?}", err);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("MQTT error: {:?}", e);
                }
            }
        }
    });

    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
