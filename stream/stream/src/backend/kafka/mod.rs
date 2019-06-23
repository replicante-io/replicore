use failure::ResultExt;
use futures::Future;
use humthreads::ThreadScope;
use rdkafka::config::ClientConfig;
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::message::OwnedHeaders;
use rdkafka::producer::FutureProducer;
use rdkafka::producer::FutureRecord;
use serde::de::DeserializeOwned;
use serde::Serialize;

use replicante_externals_kafka::ClientStatsContext;

use crate::config::KafkaConfig;
use crate::traits::StreamInterface;
use crate::EmitMessage;
use crate::ErrorKind;
use crate::Iter;
use crate::Result;
use crate::StreamOpts;

static KAFKA_STATS_INTERVAL: &'static str = "1000";

/// Generic stream backed by Kafka.
pub struct KafkaStream {
    producer: FutureProducer<ClientStatsContext>,
    timeout: i64,
    topic: String,
}

impl KafkaStream {
    pub fn new(config: KafkaConfig, opts: StreamOpts) -> Result<KafkaStream> {
        let timeout = i64::from(config.common.timeouts.request);
        let topic = format!("{}_{}", config.topic_prefix, opts.stream_id);
        let producer = KafkaStream::producer(config, opts)?;
        Ok(KafkaStream {
            producer,
            timeout,
            topic,
        })
    }

    fn producer(
        config: KafkaConfig,
        opts: StreamOpts,
    ) -> Result<FutureProducer<ClientStatsContext>> {
        let client_id = format!("stream:{}:producer", opts.stream_id);
        let client_context = ClientStatsContext::new(client_id.as_str());
        opts.healthchecks
            .register(client_id.as_str(), client_context.healthcheck());
        let mut kafka_config = ClientConfig::new();
        kafka_config
            .set("bootstrap.servers", &config.common.brokers)
            .set("client.id", &client_id)
            .set(
                "metadata.request.timeout.ms",
                &config.common.timeouts.metadata.to_string(),
            )
            .set(
                "request.timeout.ms",
                &config.common.timeouts.request.to_string(),
            )
            .set(
                "socket.timeout.ms",
                &config.common.timeouts.socket.to_string(),
            )
            .set("statistics.interval.ms", KAFKA_STATS_INTERVAL)
            .set_log_level(RDKafkaLogLevel::Debug);
        let producer = kafka_config
            .create_with_context(client_context)
            .with_context(|_| ErrorKind::BackendClientCreation)?;
        Ok(producer)
    }
}

impl<T> StreamInterface<T> for KafkaStream
where
    T: DeserializeOwned + Serialize + 'static,
{
    fn emit(&self, message: EmitMessage) -> Result<()> {
        let mut headers = OwnedHeaders::new_with_capacity(message.headers.len());
        for (key, value) in &message.headers {
            headers = headers.add(key, value);
        }
        let record: FutureRecord<String, [u8]> = FutureRecord::to(&self.topic)
            .headers(headers)
            .key(&message.id)
            .payload(&message.payload);
        self.producer
            .send(record, self.timeout)
            .wait()
            .with_context(|_| ErrorKind::EmitFailed)?
            .map_err(|(error, _)| error)
            .with_context(|_| ErrorKind::EmitFailed)?;
        Ok(())
    }

    fn follow<'a>(
        &self,
        _group: String,
        _thread: &'a ThreadScope,
        _tail: bool,
    ) -> Result<Iter<'a, T>> {
        panic!("TODO: KafkaStream::follow");
    }
}
