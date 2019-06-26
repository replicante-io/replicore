use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::unbounded;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use failure::ResultExt;
use humthreads::ThreadScope;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::iter::Backoff;
use crate::traits::MessageInterface;
use crate::traits::StreamInterface;
use crate::EmitMessage;
use crate::ErrorKind;
use crate::Iter;
use crate::Message;
use crate::Result;
use crate::Stream;

/// Generic mock Stream to write tests against.
///
/// This mock DOES NOT support multiple follower groups.
/// Every message emitted is delivered to ONLY ONE follower.
#[derive(Clone)]
pub struct MockStream {
    receiver: Receiver<SerialisedMessage>,
    sender: Sender<SerialisedMessage>,
    stream_id: &'static str,
}

impl MockStream {
    pub fn make<T>(stream_id: &'static str) -> Stream<T>
    where
        T: DeserializeOwned + Serialize + 'static,
    {
        let logger = slog::Logger::root(slog::Discard, slog::o!());
        let (sender, receiver) = unbounded();
        let stream = MockStream {
            receiver,
            sender,
            stream_id,
        };
        Stream::with_backend(stream_id, Arc::new(stream), logger, None)
    }
}

impl<T> StreamInterface<T> for MockStream
where
    T: DeserializeOwned + Serialize + 'static,
{
    fn emit(&self, message: EmitMessage<T>) -> Result<()> {
        self.sender
            .send(SerialisedMessage::serialise(message)?)
            .with_context(|_| ErrorKind::EmitFailed)?;
        Ok(())
    }

    fn follow<'a>(
        &self,
        group: String,
        thread: &'a ThreadScope,
        tail: bool,
    ) -> Result<Iter<'a, T>> {
        let stream_id: &'static str = self.stream_id;
        let iter: Box<dyn Iterator<Item = Result<Message<T>>> + 'a> = if tail {
            Box::new(ChannelIter {
                _enfoce_paylod_type: PhantomData,
                follow_id: group.clone(),
                receiver: self.receiver.clone(),
                thread,
                stream_id,
            })
        } else {
            let follow_id = group.clone();
            Box::new(self.receiver.clone().into_iter().map(move |message| {
                let message = message.deserialize(stream_id, follow_id.clone());
                Ok(message)
            }))
        };
        Ok(Iter::with_iter(
            self.stream_id,
            group,
            Backoff::fast(),
            thread,
            iter,
        ))
    }
}

struct ChannelIter<'a, T>
where
    T: DeserializeOwned + 'static,
{
    _enfoce_paylod_type: PhantomData<T>,
    follow_id: String,
    receiver: Receiver<SerialisedMessage>,
    stream_id: &'static str,
    thread: &'a ThreadScope,
}

impl<'a, T> Iterator for ChannelIter<'a, T>
where
    T: DeserializeOwned,
{
    type Item = Result<Message<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.thread.should_shutdown() {
            let message = match self.receiver.recv_timeout(Duration::from_millis(50)) {
                Ok(message) => message,
                Err(error) if error.is_timeout() => continue,
                Err(_) => return None,
            };
            let message = message.deserialize(self.stream_id, self.follow_id.clone());
            return Some(Ok(message));
        }
        None
    }
}

struct ChannelMessage {
    id: String,
}

impl MessageInterface for ChannelMessage {
    fn async_ack(&self) -> Result<()> {
        Ok(())
    }

    fn id(&self) -> &str {
        &self.id
    }
}

struct SerialisedMessage {
    headers: HashMap<String, String>,
    id: String,
    payload: Vec<u8>,
}

impl SerialisedMessage {
    fn deserialize<T>(self, stream_id: &'static str, follow_id: String) -> Message<T>
    where
        T: DeserializeOwned + 'static,
    {
        let inner = Rc::new(ChannelMessage { id: self.id });
        Message::with_backend(stream_id, follow_id, self.headers, self.payload, inner)
    }

    fn serialise<T>(message: EmitMessage<T>) -> Result<SerialisedMessage>
    where
        T: Serialize + 'static,
    {
        let payload =
            serde_json::to_vec(&message.payload).with_context(|_| ErrorKind::PayloadEncode)?;
        Ok(SerialisedMessage {
            headers: message.headers,
            id: message.id,
            payload,
        })
    }
}
