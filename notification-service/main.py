import os
import sys
import json
import smtplib
from email.mime.text import MIMEText
from kafka import KafkaConsumer
import kafka.errors

SENDER_EMAIL = os.getenv("SENDER_EMAIL", "mehmetrizaoz@gmail.com")
RECIPIENT_EMAIL = os.getenv("RECIPIENT_EMAIL", "mehmetrizaoz@gmail.com")
SMTP_PASSWORD = os.getenv("SMTP_PASSWORD", "ewoxcxuohgqybnzj")


def send_email(service_name: str, status: str):
    print(f"[EMAIL] Preparing to send email from {SENDER_EMAIL} to {RECIPIENT_EMAIL}", flush=True)

    msg = MIMEText(f"Service '{service_name}' reported status: '{status}'")
    msg["Subject"] = "IoT Service Notification"
    msg["From"] = SENDER_EMAIL
    msg["To"] = RECIPIENT_EMAIL

    try:
        with smtplib.SMTP("smtp.gmail.com", 587) as server:
            server.starttls()
            server.login(SENDER_EMAIL, SMTP_PASSWORD)
            server.send_message(msg)
        print(f"[EMAIL] Sent successfully to {RECIPIENT_EMAIL}", flush=True)
    except Exception as e:
        print(f"[EMAIL] Failed to send: {e}", flush=True)


def main():
    print("Notification Service started - listening for service notifications", flush=True)
    print(f"Connecting to Kafka at kafka:9092...", flush=True)

    try:
        consumer = KafkaConsumer(
            "service-notifications",
            bootstrap_servers="kafka:9092",
            group_id="notification-service-group",
            auto_offset_reset="earliest",
            enable_auto_commit=True,
            value_deserializer=lambda m: json.loads(m.decode("utf-8")),
        )
        print("Connected to Kafka successfully!", flush=True)
    except Exception as e:
        print(f"Failed to connect to Kafka: {e}", flush=True)
        sys.exit(1)

    for message in consumer:
        msg = message.value
        print(f"[NOTIFICATION] Service '{msg['service_name']}' reported status: '{msg['status']}'", flush=True)
        send_email(msg["service_name"], msg["status"])


if __name__ == "__main__":
    main()
