import paho.mqtt.client as mqtt
import random
import json
import time
import threading
from datetime import datetime

MQTT_BROKER = "mqtt-broker"
MQTT_PORT = 1883
DEVICE_ID = "sensor-001"

CONTROL_TOPIC = "control/command"

class SensorSimulator:
    def __init__(self):
        self.settings = {
            "temperature": {"min": 15.0, "max": 35.0, "interval": 7},
            "velocity": {"min": 1000.0, "max": 5000.0, "interval": 6},
            "position": {"x_range": 100.0, "y_range": 100.0, "z_range": 50.0, "interval": 5}
        }
        self.lock = threading.Lock()

    def update_settings(self, sensor_type, new_settings):
        with self.lock:
            if sensor_type == "temperature":
                self.settings["temperature"]["min"] = new_settings.get("min", self.settings["temperature"]["min"])
                self.settings["temperature"]["max"] = new_settings.get("max", self.settings["temperature"]["max"])
                self.settings["temperature"]["interval"] = new_settings.get("interval", self.settings["temperature"]["interval"])
            elif sensor_type == "velocity":
                self.settings["velocity"]["min"] = new_settings.get("min", self.settings["velocity"]["min"])
                self.settings["velocity"]["max"] = new_settings.get("max", self.settings["velocity"]["max"])
                self.settings["velocity"]["interval"] = new_settings.get("interval", self.settings["velocity"]["interval"])
            elif sensor_type == "position":
                self.settings["position"]["x_range"] = new_settings.get("x_range", self.settings["position"]["x_range"])
                self.settings["position"]["y_range"] = new_settings.get("y_range", self.settings["position"]["y_range"])
                self.settings["position"]["z_range"] = new_settings.get("z_range", self.settings["position"]["z_range"])
                self.settings["position"]["interval"] = new_settings.get("interval", self.settings["position"]["interval"])
            print(f"[{datetime.now().strftime('%Y-%m-%d %H:%M:%S')}] SETTINGS UPDATED: {sensor_type} -> {self.settings[sensor_type]}")

def on_connect(client, userdata, flags, rc):
    print(f"[{datetime.now().strftime('%Y-%m-%d %H:%M:%S')}] Connected to MQTT broker with result code {rc}")

def on_message(client, userdata, msg):
    try:
        if msg.topic.startswith("sensors/settings/"):
            sensor_type = msg.topic.split("/")[-1]
            settings = json.loads(msg.payload.decode())
            simulator.update_settings(sensor_type, settings)
        elif msg.topic == CONTROL_TOPIC:
            payload = json.loads(msg.payload.decode())
            print(f"[{datetime.now().strftime('%Y-%m-%d %H:%M:%S')}] RECEIVED: topic={msg.topic}, payload={payload}")
    except Exception as e:
        print(f"[{datetime.now().strftime('%Y-%m-%d %H:%M:%S')}] RECEIVED: topic={msg.topic}, error={e}")

simulator = SensorSimulator()

def publish_temperature(client):
    while True:
        with simulator.lock:
            min_val = simulator.settings["temperature"]["min"]
            max_val = simulator.settings["temperature"]["max"]
            interval = simulator.settings["temperature"]["interval"]
        
        temperature = round(random.uniform(min_val, max_val), 1)
        timestamp = int(time.time())
        payload = {
            "device_id": DEVICE_ID,
            "sensor_type": "temperature",
            "value": temperature,
            "unit": "celsius",
            "timestamp": timestamp
        }
        client.publish("sensors/temperature", json.dumps(payload))
        print(f"[{datetime.now().strftime('%Y-%m-%d %H:%M:%S')}] SENT: temp={temperature}°C -> topic: sensors/temperature")
        time.sleep(interval)

def publish_velocity(client):
    while True:
        with simulator.lock:
            min_val = simulator.settings["velocity"]["min"]
            max_val = simulator.settings["velocity"]["max"]
            interval = simulator.settings["velocity"]["interval"]
        
        rpm = round(random.uniform(min_val, max_val), 1)
        timestamp = int(time.time())
        payload = {
            "device_id": DEVICE_ID,
            "sensor_type": "velocity",
            "value": rpm,
            "unit": "rpm",
            "timestamp": timestamp
        }
        client.publish("sensors/velocity", json.dumps(payload))
        print(f"[{datetime.now().strftime('%Y-%m-%d %H:%M:%S')}] SENT: velocity={rpm} rpm -> topic: sensors/velocity")
        time.sleep(interval)

def publish_position(client):
    while True:
        with simulator.lock:
            x_range = simulator.settings["position"]["x_range"]
            y_range = simulator.settings["position"]["y_range"]
            z_range = simulator.settings["position"]["z_range"]
            interval = simulator.settings["position"]["interval"]
        
        x = round(random.uniform(-x_range, x_range), 2)
        y = round(random.uniform(-y_range, y_range), 2)
        z = round(random.uniform(-z_range, z_range), 2)
        timestamp = int(time.time())
        payload = {
            "device_id": DEVICE_ID,
            "sensor_type": "position",
            "value": {"x": x, "y": y, "z": z},
            "unit": "meters",
            "timestamp": timestamp
        }
        client.publish("sensors/position", json.dumps(payload))
        print(f"[{datetime.now().strftime('%Y-%m-%d %H:%M:%S')}] SENT: position=({x}, {y}, {z})m -> topic: sensors/position")
        time.sleep(interval)

def main():
    client = mqtt.Client()
    client.on_connect = on_connect
    client.on_message = on_message
    
    client.connect(MQTT_BROKER, MQTT_PORT, 60)
    client.loop_start()
    
    client.subscribe(CONTROL_TOPIC)
    client.subscribe("sensors/settings/#")
    
    print(f"[{datetime.now().strftime('%Y-%m-%d %H:%M:%S')}] Sensor Simulator started")
    print(f"[{datetime.now().strftime('%Y-%m-%d %H:%M:%S')}] Subscribed to: {CONTROL_TOPIC}, sensors/settings/#")
    
    threads = [
        threading.Thread(target=publish_temperature, args=(client,)),
        threading.Thread(target=publish_velocity, args=(client,)),
        threading.Thread(target=publish_position, args=(client,)),
    ]
    
    for t in threads:
        t.daemon = True
        t.start()
    
    for t in threads:
        t.join()

if __name__ == "__main__":
    main()
