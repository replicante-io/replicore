use std::marker::PhantomData;
use std::rc::Rc;
use std::time::Duration;

use failure::ResultExt;
use humthreads::ThreadScope;
use rdkafka::consumer::Consumer;
use rdkafka::message::BorrowedMessage;
use rdkafka::message::Message as KMessage;
use rdkafka::topic_partition_list::Offset;
use rdkafka::topic_partition_list::TopicPartitionList;
use serde::de::DeserializeOwned;

use replicante_externals_kafka::headers_to_map;

use super::client::StatsConsumer;
use crate::traits::MessageInterface;
use crate::Error;
use crate::ErrorKind;
use crate::Message;
use crate::Result;

/// Iterator following the stream for a group.
pub struct KafkaIter<'a, T>
where
    T: DeserializeOwned + 'static,
{
    _enfoce_paylod_type: PhantomData<T>,
    consumer: Rc<StatsConsumer>,
    follow_id: String,
    stream_id: &'static str,
    stream_started: bool,
    tail: bool,
    thread: Option<&'a ThreadScope>,
}

impl<'a, T> KafkaIter<'a, T>
where
    T: DeserializeOwned,
{
    pub fn new(
        consumer: StatsConsumer,
        follow_id: String,
        stream_id: &'static str,
        tail: bool,
        thread: Option<&'a ThreadScope>,
    ) -> KafkaIter<'a, T> {
        let consumer = Rc::new(consumer);
        KafkaIter {
            _enfoce_paylod_type: PhantomData,
            consumer,
            follow_id,
            stream_id,
            stream_started: false,
            tail,
            thread,
        }
    }
}

impl<'a, T> Iterator for KafkaIter<'a, T>
where
    T: DeserializeOwned,
{
    type Item = Result<Message<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.thread.map(|t| t.should_shutdown()).unwrap_or(false) {
            let record = match self.consumer.poll(Duration::from_millis(500)) {
                None if self.tail => continue,
                // Streams take some time to start receiving messages from kafka.
                // If we are not tailing the stream, keep checking until we get at least
                // one message, after that we can return at the first empty poll.
                None if !self.stream_started => continue,
                None => return None,
                Some(Ok(record)) => record,
                Some(Err(error)) => {
                    let error = Err(error)
                        .with_context(|_| ErrorKind::FollowFailed)
                        .map_err(Error::from);
                    return Some(error);
                }
            };
            self.stream_started = true;
            return Some(KafkaMessage::decode(
                Rc::clone(&self.consumer),
                self.stream_id,
                self.follow_id.clone(),
                record,
            ));
        }
        None
    }
}

struct KafkaMessage {
    consumer: Rc<StatsConsumer>,
    kafka_id: String,
    offset: i64,
    partition: i32,
    topic: String,
}

impl KafkaMessage {
    fn decode<'a, T>(
        consumer: Rc<StatsConsumer>,
        stream_id: &'static str,
        follow_id: String,
        record: BorrowedMessage<'a>,
    ) -> Result<Message<T>>
    where
        T: DeserializeOwned + 'static,
    {
        let kafka_id = format!(
            "topic={};partition={};offset={}",
            record.topic(),
            record.partition(),
            record.offset(),
        );
        let inner = Rc::new(KafkaMessage {
            consumer,
            kafka_id,
            offset: record.offset(),
            partition: record.partition(),
            topic: record.topic().to_string(),
        });
        let headers = headers_to_map(record.headers())
            .with_context(|e| ErrorKind::MessageInvalidHeader(e.header.clone()))?;
        let payload = match record.payload() {
            Some(payload) => payload.to_vec(),
            None => return Err(ErrorKind::MessageNoPayload.into()),
        };
        Ok(Message::with_backend(
            stream_id, follow_id, headers, payload, inner,
        ))
    }
}

impl MessageInterface for KafkaMessage {
    fn async_ack(&self) -> Result<()> {
        // Kafka needs us to commit the offset of the NEXT message to FETCH,
        // not the offset of the last message processed.
        let offset = Offset::Offset(self.offset + 1);
        let mut request = TopicPartitionList::with_capacity(1);
        request.add_partition_offset(&self.topic, self.partition, offset);
        self.consumer
            .store_offsets(&request)
            .with_context(|_| ErrorKind::AckFailed)?;
        Ok(())
    }

    fn id(&self) -> &str {
        &self.kafka_id
    }
}
