FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    openjdk-11-jre-headless \
    wget \
    curl \
    netcat \
    && rm -rf /var/lib/apt/lists/*

ARG KAFKA_VERSION=3.6.1
ARG SCALA_VERSION=2.13
ARG KAFKA_HOME=/opt/kafka

RUN wget -q https://archive.apache.org/dist/kafka/${KAFKA_VERSION}/kafka_2.13-${KAFKA_VERSION}.tgz && \
    tar -xzf kafka_2.13-${KAFKA_VERSION}.tgz -C /opt && \
    rm kafka_2.13-${KAFKA_VERSION}.tgz && \
    ln -s /opt/kafka_2.13-${KAFKA_VERSION} ${KAFKA_HOME}

ENV KAFKA_HOME=${KAFKA_HOME}
ENV PATH=${PATH}:${KAFKA_HOME}/bin

WORKDIR ${KAFKA_HOME}

COPY kafka-entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/kafka-entrypoint.sh

EXPOSE 9092

ENTRYPOINT ["kafka-entrypoint.sh"]
