use std::time::Duration;

use failure::ResultExt;
use humthreads::ThreadScope;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::Consumer;
use rdkafka::producer::FutureProducer;
use rdkafka::producer::FutureRecord;
use serde::de::DeserializeOwned;
use serde::Serialize;

use replicante_externals_kafka::headers_from_map;
use replicante_externals_kafka::ClientStatsContext;
use replicante_externals_kafka::KafkaHealthChecker;

use crate::config::KafkaConfig;
use crate::iter::Backoff;
use crate::traits::StreamInterface;
use crate::EmitMessage;
use crate::ErrorKind;
use crate::Iter;
use crate::Result;
use crate::StreamOpts;

mod client;
mod iter;

use self::iter::KafkaIter;

/// Generic stream backed by Kafka.
pub struct KafkaStream {
    consumers_config: ClientConfig,
    consumers_health: KafkaHealthChecker,
    producer: FutureProducer<ClientStatsContext>,
    producer_timeout: Duration,
    topic: String,
    stream_id: &'static str,
}

impl KafkaStream {
    pub fn new(config: KafkaConfig, opts: StreamOpts) -> Result<KafkaStream> {
        let stream_id = opts.stream_id;
        let healthchecks = opts.healthchecks;

        // Attributes used to generate consumers to follow streams.
        let consumers_config = client::consumers_config(&config);
        let consumers_health = KafkaHealthChecker::new();
        let consumers_health_id = format!("stream:{}:followers", stream_id);
        healthchecks.register(consumers_health_id, consumers_health.clone());

        // Producer instance used to emit messages.
        let producer = client::producer(&config, stream_id, healthchecks)?;
        let producer_timeout = Duration::from_secs(config.common.timeouts.request);
        let topic = format!("{}_{}", config.topic_prefix, stream_id);

        Ok(KafkaStream {
            consumers_config,
            consumers_health,
            producer,
            producer_timeout,
            topic,
            stream_id,
        })
    }
}

impl<T> StreamInterface<T> for KafkaStream
where
    T: DeserializeOwned + Serialize + 'static,
{
    fn emit(&self, message: EmitMessage<T>) -> Result<()> {
        let headers = headers_from_map(&message.headers);
        let record: FutureRecord<String, [u8]> = FutureRecord::to(&self.topic)
            .headers(headers)
            .key(&message.id)
            .payload(&message.payload);
        let send = self.producer.send(record, self.producer_timeout);
        futures::executor::block_on(send)
            .map_err(|(error, _)| error)
            .with_context(|_| ErrorKind::EmitFailed)?;
        Ok(())
    }

    fn follow<'a>(
        &self,
        group: String,
        thread: Option<&'a ThreadScope>,
        tail: bool,
    ) -> Result<Iter<'a, T>> {
        let consumer = client::consumer(
            self.consumers_config.clone(),
            self.consumers_health.clone(),
            self.stream_id,
            &group,
        )?;
        consumer
            .subscribe(&[&self.topic])
            .with_context(|_| ErrorKind::BackendClientCreation)?;
        let iter = Box::new(KafkaIter::new(
            consumer,
            group.clone(),
            self.stream_id,
            tail,
            thread,
        ));
        Ok(Iter::with_iter(
            self.stream_id,
            group,
            Backoff::new(),
            thread,
            iter,
        ))
    }
}
