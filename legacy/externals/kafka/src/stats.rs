use std::sync::Arc;
use std::sync::Mutex;

use rdkafka::consumer::ConsumerContext;
use rdkafka::statistics::Statistics;
use rdkafka::ClientContext;

use replicante_models_api::HealthStatus;
use replicante_service_healthcheck::HealthCheck;

use crate::metrics::*;

/// A kafka client context to expose stats over prometheus.
pub struct ClientStatsContext {
    health: KafkaHealthChecker,
    role: String,
}

impl ClientStatsContext {
    pub fn new<S>(role: S) -> ClientStatsContext
    where
        S: Into<String>,
    {
        let health = KafkaHealthChecker::new();
        ClientStatsContext::with_healthcheck(role, health)
    }

    /// Return an `HealthCheck` that reports on the state of this client.
    pub fn healthcheck(&self) -> KafkaHealthChecker {
        self.health.clone()
    }

    pub fn with_healthcheck<S>(role: S, health: KafkaHealthChecker) -> ClientStatsContext
    where
        S: Into<String>,
    {
        let role = role.into();
        ClientStatsContext { health, role }
    }
}

impl ClientContext for ClientStatsContext {
    fn stats(&self, stats: Statistics) {
        KAFKA_MSG_CNT
            .with_label_values(&[&self.role])
            .set(stats.msg_cnt as f64);
        KAFKA_REPLYQ
            .with_label_values(&[&self.role])
            .set(stats.replyq as f64);
        let mut brokers_down = 0;
        let mut brokers_other = 0;
        let mut brokers_up = 0;
        for broker in stats.brokers.values() {
            match broker.state.as_str() {
                "DOWN" => brokers_down += 1,
                "UP" => brokers_up += 1,
                _ => brokers_other += 1,
            };
            KAFKA_BROKER_OUTBUF_CNT
                .with_label_values(&[&self.role, &broker.name])
                .set(broker.outbuf_cnt as f64);
            KAFKA_BROKER_OUTBUF_MSG_CNT
                .with_label_values(&[&self.role, &broker.name])
                .set(broker.outbuf_msg_cnt as f64);
            KAFKA_BROKER_REQ_TIMEOUTS
                .with_label_values(&[&self.role, &broker.name])
                .set(broker.req_timeouts as f64);
            KAFKA_BROKER_RX
                .with_label_values(&[&self.role, &broker.name])
                .set(broker.rx as f64);
            KAFKA_BROKER_RXBYTES
                .with_label_values(&[&self.role, &broker.name])
                .set(broker.rxbytes as f64);
            KAFKA_BROKER_RXCORRIDERRS
                .with_label_values(&[&self.role, &broker.name])
                .set(broker.rxcorriderrs as f64);
            KAFKA_BROKER_RXERRS
                .with_label_values(&[&self.role, &broker.name])
                .set(broker.rxerrs as f64);
            KAFKA_BROKER_RXPARTIAL
                .with_label_values(&[&self.role, &broker.name])
                .set(broker.rxpartial as f64);
            KAFKA_BROKER_TX
                .with_label_values(&[&self.role, &broker.name])
                .set(broker.tx as f64);
            KAFKA_BROKER_TXBYTES
                .with_label_values(&[&self.role, &broker.name])
                .set(broker.txbytes as f64);
            KAFKA_BROKER_TXERRS
                .with_label_values(&[&self.role, &broker.name])
                .set(broker.txerrs as f64);
            KAFKA_BROKER_TXRETRIES
                .with_label_values(&[&self.role, &broker.name])
                .set(broker.txretries as f64);
            KAFKA_BROKER_WAITRESP_CNT
                .with_label_values(&[&self.role, &broker.name])
                .set(broker.waitresp_cnt as f64);
            KAFKA_BROKER_WAITRESP_MSG_CNT
                .with_label_values(&[&self.role, &broker.name])
                .set(broker.waitresp_msg_cnt as f64);
            KAFKA_BROKER_WAKEUPS
                .with_label_values(&[&self.role, &broker.name])
                .set(broker.wakeups.unwrap_or(0) as f64);
            KAFKA_BROKER_ZBUF_GROW
                .with_label_values(&[&self.role, &broker.name])
                .set(broker.zbuf_grow as f64);
        }
        for topic in stats.topics.values() {
            for partition in topic.partitions.values() {
                let id = partition.partition.to_string();
                KAFKA_PARTITION_COMMITTED_OFFSET
                    .with_label_values(&[&self.role, &topic.topic, &id])
                    .set(partition.committed_offset as f64);
                KAFKA_PARTITION_CONSUMER_LAG
                    .with_label_values(&[&self.role, &topic.topic, &id])
                    .set(partition.consumer_lag as f64);
                KAFKA_PARTITION_FETCHQ_CNT
                    .with_label_values(&[&self.role, &topic.topic, &id])
                    .set(partition.fetchq_cnt as f64);
                KAFKA_PARTITION_FETCHQ_SIZE
                    .with_label_values(&[&self.role, &topic.topic, &id])
                    .set(partition.fetchq_size as f64);
                KAFKA_PARTITION_MSGQ_BYTES
                    .with_label_values(&[&self.role, &topic.topic, &id])
                    .set(partition.msgq_bytes as f64);
                KAFKA_PARTITION_MSGQ_CNT
                    .with_label_values(&[&self.role, &topic.topic, &id])
                    .set(partition.msgq_cnt as f64);
                KAFKA_PARTITION_TXBYTES
                    .with_label_values(&[&self.role, &topic.topic, &id])
                    .set(partition.txbytes as f64);
                KAFKA_PARTITION_TXMSGS
                    .with_label_values(&[&self.role, &topic.topic, &id])
                    .set(partition.txmsgs as f64);
                KAFKA_PARTITION_XMIT_MSGQ_BYTES
                    .with_label_values(&[&self.role, &topic.topic, &id])
                    .set(partition.xmit_msgq_bytes as f64);
                KAFKA_PARTITION_XMIT_MSGQ_CNT
                    .with_label_values(&[&self.role, &topic.topic, &id])
                    .set(partition.xmit_msgq_cnt as f64);
            }
        }
        if let Some(cgrp) = stats.cgrp {
            KAFKA_CGRP_ASSIGNMENT_SIZE
                .with_label_values(&[&self.role])
                .set(f64::from(cgrp.assignment_size));
        }
        // Summarize kafka health based on brokers state.
        let health = if brokers_up == 0 {
            HealthStatus::Failed("all kafka brokers are down".into())
        } else if brokers_down > 0 {
            HealthStatus::Degraded("some kafka brokers are down".into())
        } else if brokers_other > 0 {
            HealthStatus::Degraded("not all kafka brokers are up".into())
        } else {
            HealthStatus::Healthy
        };
        self.health.update(health);
    }
}

impl ConsumerContext for ClientStatsContext {}

/// HealthCheck implementation to report kafka health status.
#[derive(Clone)]
pub struct KafkaHealthChecker {
    health: Arc<Mutex<HealthStatus>>,
}

impl KafkaHealthChecker {
    pub fn new() -> KafkaHealthChecker {
        let health = Arc::new(Mutex::new(HealthStatus::Degraded(
            "initialising kafka client".to_string(),
        )));
        KafkaHealthChecker { health }
    }

    fn update(&self, health: HealthStatus) {
        *self
            .health
            .lock()
            .expect("KafkaHealthChecker mutex poisoned") = health;
    }
}

impl HealthCheck for KafkaHealthChecker {
    fn check(&self) -> HealthStatus {
        self.health
            .lock()
            .expect("KafkaHealthChecker mutex poisoned")
            .clone()
    }
}

impl Default for KafkaHealthChecker {
    fn default() -> KafkaHealthChecker {
        KafkaHealthChecker::new()
    }
}
