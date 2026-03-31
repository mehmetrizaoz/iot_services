#!/bin/bash
set -e

if [ ! -f /tmp/kraft-combined-logs/meta.properties ]; then
    $KAFKA_HOME/bin/kafka-storage.sh random-uuid > /tmp/cluster-id

    sed -i 's/^advertised.listeners=.*/advertised.listeners=PLAINTEXT:\/\/kafka:9092/' $KAFKA_HOME/config/kraft/server.properties

    $KAFKA_HOME/bin/kafka-storage.sh format -t $(cat /tmp/cluster-id) -c $KAFKA_HOME/config/kraft/server.properties
fi

$KAFKA_HOME/bin/kafka-server-start.sh $KAFKA_HOME/config/kraft/server.properties

tail -f /dev/null
