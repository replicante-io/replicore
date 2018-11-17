use prometheus::GaugeVec;
use prometheus::Opts;
use prometheus::Registry;

use rdkafka::ClientContext;
use rdkafka::consumer::ConsumerContext;
use rdkafka::statistics::Statistics;
use slog::Logger;


lazy_static! {
    pub static ref KAFKA_BROKER_OUTBUF_CNT: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_broker_outbuf_cnt",
            "Number of requests awaiting transmission to broker"
        ),
        &["role", "broker"]
    ).expect("Failed to create KAFKA_BROKER_OUTBUF_CNT gauge");

    pub static ref KAFKA_BROKER_OUTBUF_MSG_CNT: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_broker_outbuf_msg_cnt",
            "Number of messages awaiting transmission to broker"
        ),
        &["role", "broker"]
    ).expect("Failed to create KAFKA_BROKER_OUTBUF_MSG_CNT gauge");

    // TODO: figure out how to reset the states (enum?)
    //pub static ref KAFKA_BROKER_STATE: GaugeVec = GaugeVec::new(
    //    Opts::new(
    //        "replicore_tasks_kafka_broker_state",
    //        "Broker state"
    //    ),
    //    &["role", "broker", "state"]
    //).expect("Failed to create KAFKA_BROKER_STATE gauge");

    pub static ref KAFKA_BROKER_TX: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_broker_tx",
            "Total number of requests sent"
        ),
        &["role", "broker"]
    ).expect("Failed to create KAFKA_BROKER_TX gauge");

    pub static ref KAFKA_BROKER_WAITRESP_CNT: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_broker_waitresp_cnt",
            "Number of requests in-flight to broker awaiting response"
        ),
        &["role", "broker"]
    ).expect("Failed to create KAFKA_BROKER_WAITRESP_CNT gauge");

    pub static ref KAFKA_BROKER_WAITRESP_MSG_CNT: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_broker_waitresp_msg_cnt",
            "Number of messages in-flight to broker awaitign response"
        ),
        &["role", "broker"]
    ).expect("Failed to create KAFKA_BROKER_WAITRESP_MSG_CNT gauge");

    pub static ref KAFKA_MSG_CNT: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_msg_cnt",
            "Current number of messages in producer queues"
        ),
        &["role"]
    ).expect("Failed to create KAFKA_MSG_CNT gauge");

    pub static ref KAFKA_REPLYQ: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_replyq",
            "Number of ops waiting in queue for application to serve with rd_kafka_poll()"
        ),
        &["role"]
    ).expect("Failed to create KAFKA_REPLYQ_GAUGE gauge");
}


/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    if let Err(err) = registry.register(Box::new(KAFKA_BROKER_OUTBUF_CNT.clone())) {
        debug!(logger, "Failed to register KAFKA_BROKER_OUTBUF_CNT"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_BROKER_OUTBUF_MSG_CNT.clone())) {
        debug!(logger, "Failed to register KAFKA_BROKER_OUTBUF_MSG_CNT"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_BROKER_TX.clone())) {
        debug!(logger, "Failed to register KAFKA_BROKER_TX"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_BROKER_WAITRESP_CNT.clone())) {
        debug!(logger, "Failed to register KAFKA_BROKER_WAITRESP_CNT"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_BROKER_WAITRESP_MSG_CNT.clone())) {
        debug!(logger, "Failed to register KAFKA_BROKER_WAITRESP_MSG_CNT"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_MSG_CNT.clone())) {
        debug!(logger, "Failed to register KAFKA_MSG_CNT"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_REPLYQ.clone())) {
        debug!(logger, "Failed to register KAFKA_REPLYQ"; "error" => ?err);
    }
}


/// A kafka client context to expose stats over prometheus.
pub struct ClientStatsContext {
    role: String,
}

impl ClientStatsContext {
    pub fn new<S: Into<String>>(role: S) -> ClientStatsContext {
        let role = role.into();
        ClientStatsContext { role, }
    }
}

impl ClientContext for ClientStatsContext {
    fn stats(&self, stats: Statistics) {
        KAFKA_MSG_CNT.with_label_values(&[&self.role]).set(stats.msg_cnt as f64);
        KAFKA_REPLYQ.with_label_values(&[&self.role]).set(stats.replyq as f64);
        for broker in stats.brokers.values() {
            //KAFKA_BROKER_STATE.with_label_values(&[&self.role, &broker.name, &broker.state]).set(1);
            KAFKA_BROKER_OUTBUF_CNT.with_label_values(&[&self.role, &broker.name])
                .set(broker.outbuf_cnt as f64);
            KAFKA_BROKER_OUTBUF_MSG_CNT.with_label_values(&[&self.role, &broker.name])
                .set(broker.outbuf_msg_cnt as f64);
            KAFKA_BROKER_TX.with_label_values(&[&self.role, &broker.name]).set(broker.tx as f64);
            KAFKA_BROKER_WAITRESP_CNT.with_label_values(&[&self.role, &broker.name])
                .set(broker.waitresp_cnt as f64);
            KAFKA_BROKER_WAITRESP_MSG_CNT.with_label_values(&[&self.role, &broker.name])
                .set(broker.waitresp_msg_cnt as f64);
        }
    }
}

impl ConsumerContext for ClientStatsContext {}
