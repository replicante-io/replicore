#!/bin/bash
set -ex

CONF_ROOT=./devtools/configs/essential
DATA_ROOT=./devtools/data/essential
POD_NAME=replideps-essential
PODMAN_HOST=$HOSTNAME


# Initialise data directories
mkdir -p $DATA_ROOT/{kafka,mongo,zoo-data,zoo-log}


# Create the pod and expose ports.
podman pod create --name $POD_NAME \
  --publish '2181:2181' \
  --publish '8090:80' \
  --publish '9092:9092' \
  --publish '27017:27017'


# Start containers in dependency ortder.
# The first will "start" the pod, the others will be added.
# ==> MongoDB - for persisted state
podman run \
  --pod $POD_NAME --name "${POD_NAME}-mongo" \
  --detach --init --tty \
  --volume "${DATA_ROOT}/mongo:/data/db:z" \
  --mount "type=bind,src=${CONF_ROOT}/mongo,dst=/replicore,relabel=private" \
  mongo:4.2 \
  mongod --replSet replistore --bind_ip 0.0.0.0

# ==> Zookeeper - for both kafka and replicante
podman run \
  --pod $POD_NAME --name "${POD_NAME}-zookeeper" \
  --detach --init --tty \
  --volume "${DATA_ROOT}/zoo-data:/data:z" \
  --volume "${DATA_ROOT}/zoo-log:/datalog:z" \
  --mount "type=bind,src=${CONF_ROOT}/zookeeper.conf,dst=/conf/zoo.cfg,relabel=private" \
  --env 'ZOO_MY_ID=1' \
  zookeeper:3.5

# ==> Kafka - for tasks and streams
podman run \
  --pod $POD_NAME --name "${POD_NAME}-kafka" \
  --detach --init --tty \
  --volume "${DATA_ROOT}/kafka:/kafka:z" \
  --env 'JMX_PORT=9999' \
  --env 'KAFKA_BROKER_ID=1' \
  --env 'KAFKA_LISTENERS=PLAINTEXT://127.0.0.1:9092' \
  --env 'KAFKA_LOG_FLUSH_SCHEDULER_INTERVAL_MS=60000' \
  --env 'KAFKA_NUM_PARTITIONS=4' \
  --env 'KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR=1' \
  --env "KAFKA_ZOOKEEPER_CONNECT=${PODMAN_HOST}:2181/kafka" \
  --env "KAFKA_ADVERTISED_HOST_NAME=${PODMAN_HOST}" \
  wurstmeister/kafka:2.12-2.4.0

# ==> Nginx - to serve static pages (links index, static discovery, etc ...)
podman run \
  --pod $POD_NAME --name "${POD_NAME}-nginx" \
  --detach --init --tty \
  --mount "type=bind,src=${CONF_ROOT}/static,dst=/usr/share/nginx/html,relabel=private,ro=true" \
  nginx:1.17
