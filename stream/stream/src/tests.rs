use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use humthreads::test_support::MockThreadScope;
use humthreads::ThreadScope;

use crate::iter::Backoff;
use crate::traits::MessageInterface;
use crate::traits::StreamInterface;
use crate::EmitMessage;
use crate::Iter;
use crate::Message;
use crate::Result;
use crate::Stream;

struct VecMessage {
    id: String,
}

impl MessageInterface for VecMessage {
    fn async_ack(&self) -> Result<()> {
        Ok(())
    }

    fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Clone)]
struct VecStream {
    pub messages: Arc<Mutex<Vec<EmitMessage>>>,
    pub stream_id: &'static str,
}

impl VecStream {
    fn new() -> VecStream {
        let messages = Arc::new(Mutex::new(Vec::new()));
        let stream_id = "stream_tests";
        VecStream {
            messages,
            stream_id,
        }
    }
}

impl StreamInterface<String> for VecStream {
    fn emit(&self, message: EmitMessage) -> Result<()> {
        self.messages.lock().unwrap().push(message);
        Ok(())
    }

    fn follow<'a>(
        &self,
        group: String,
        thread: &'a ThreadScope,
        _tail: bool,
    ) -> Result<Iter<'a, String>> {
        let messages = self.messages.lock().unwrap().clone();
        let stream_id: &'static str = self.stream_id;
        let follow_id = group.clone();
        let iter = messages.into_iter().map(move |message| {
            let id = message.id;
            let headers = message.headers;
            let payload = message.payload.into();
            let backend = Rc::new(VecMessage { id });
            Ok(Message::with_backend(
                stream_id,
                follow_id.clone(),
                headers,
                payload,
                backend,
            ))
        });
        Ok(Iter::with_iter(
            self.stream_id,
            group.clone(),
            Backoff::fast(),
            thread,
            Box::new(iter),
        ))
    }
}

#[test]
fn emit() {
    let backend = VecStream::new();
    let stream = Stream::with_backend(backend.stream_id, Arc::new(backend.clone()));
    stream
        .emit(EmitMessage::with("key", "value").unwrap())
        .unwrap();
    stream.emit(EmitMessage::with("a", "b").unwrap()).unwrap();
    let messages = backend.messages.lock().unwrap().clone();
    assert_eq!(
        messages,
        vec![
            EmitMessage::with("key", "value").unwrap(),
            EmitMessage::with("a", "b").unwrap(),
        ]
    );
}

#[test]
fn follow_moves_on_after_ack() {
    let backend = VecStream::new();
    let stream = Stream::with_backend(backend.stream_id, Arc::new(backend));
    stream
        .emit(EmitMessage::with("key", "value").unwrap())
        .unwrap();
    stream.emit(EmitMessage::with("a", "b").unwrap()).unwrap();
    let scope = MockThreadScope::new().scope();
    let mut iter = stream.follow("test", &scope).unwrap();
    let m1 = iter.next().unwrap().unwrap();
    let payload1 = m1.payload().unwrap();
    m1.async_ack().unwrap();
    let m2 = iter.next().unwrap().unwrap();
    let payload2 = m2.payload().unwrap();
    m2.async_ack().unwrap();
    assert_eq!(payload1, "value");
    assert_eq!(payload2, "b");
    assert!(iter.next().is_none(), "received unexpected message");
}

#[test]
fn follow_repeats_on_retry() {
    let backend = VecStream::new();
    let stream = Stream::with_backend(backend.stream_id, Arc::new(backend));
    stream
        .emit(EmitMessage::with("key", "value").unwrap())
        .unwrap();
    stream.emit(EmitMessage::with("a", "b").unwrap()).unwrap();
    let scope = MockThreadScope::new().scope();
    let mut iter = stream.follow("test", &scope).unwrap();
    let m1 = iter.next().unwrap().unwrap();
    let payload1 = m1.payload().unwrap();
    m1.retry();
    let m2 = iter.next().unwrap().unwrap();
    assert_eq!(payload1, m2.payload().unwrap());
}

#[test]
fn follow_repeats_unacked_message() {
    let backend = VecStream::new();
    let stream = Stream::with_backend(backend.stream_id, Arc::new(backend));
    stream
        .emit(EmitMessage::with("key", "value").unwrap())
        .unwrap();
    stream.emit(EmitMessage::with("a", "b").unwrap()).unwrap();
    let scope = MockThreadScope::new().scope();
    let mut iter = stream.follow("test", &scope).unwrap();
    let m1 = iter.next().unwrap().unwrap();
    let m2 = iter.next().unwrap().unwrap();
    assert_eq!(m1.payload().unwrap(), m2.payload().unwrap());
}
