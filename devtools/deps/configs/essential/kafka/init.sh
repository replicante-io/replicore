#!/bin/bash
set -e

# Topics must be initialised before clients can follow them.
create_topic() {
  name=$1
  echo "Creating topic ${name} if needed"
  JMX_PORT='' kafka-topics.sh --create --zookeeper ${KAFKA_ZOOKEEPER_CONNECT} \
    --if-not-exists --topic ${name} --partitions 3 --replication-factor 1
}

create_topic 'stream_events'
create_topic 'task_discover_clusters'
create_topic 'task_discover_clusters_retry'
create_topic 'task_orchestrate_cluster'
create_topic 'task_orchestrate_cluster_retry'
