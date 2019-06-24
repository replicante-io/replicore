use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

use failure::ResultExt;
use opentracingrust::ExtractFormat;
use opentracingrust::InjectFormat;
use opentracingrust::Result as OTResult;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::metrics::ACK_ERROR;
use crate::metrics::ACK_TOTAL;
use crate::traits::MessageInterface;
use crate::Error;
use crate::ErrorKind;
use crate::Result;

/// Wrap metadata and payload to emit to a stream.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct EmitMessage {
    pub(crate) id: String,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) payload: Vec<u8>,
}

impl EmitMessage {
    /// Create an `EmitMessage` request with the given message ID and payload.
    pub fn with<S, T>(id: S, payload: T) -> Result<EmitMessage>
    where
        S: Into<String>,
        T: Serialize,
    {
        let payload = serde_json::to_vec(&payload).with_context(|_| ErrorKind::PayloadEncode)?;
        let id = id.into();
        let headers = HashMap::new();
        Ok(EmitMessage {
            id,
            headers,
            payload,
        })
    }

    /// Attach or update an header to the message.
    pub fn header<S1, S2>(&mut self, header: S1, value: S2)
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        self.headers.insert(header.into(), value.into());
    }

    /// Attach or update a set of headers to the message.
    pub fn headers(&mut self, headers: HashMap<String, String>) {
        for (key, value) in headers {
            self.headers.insert(key, value);
        }
    }

    /// Inject a span context into the task request.
    ///
    /// Tasks handlers can then extract the span context to provide a full
    /// trace of the larger task across processes/systems.
    pub fn trace(&mut self, context: &SpanContext, tracer: &Tracer) -> OTResult<()> {
        let mut headers = HashMap::new();
        let format = InjectFormat::HttpHeaders(Box::new(&mut headers));
        tracer.inject(context, format)?;
        self.headers(headers);
        Ok(())
    }
}

/// Wrap metadata and payload received while following a stream.
pub struct Message<T>
where
    T: DeserializeOwned + 'static,
{
    _enfoce_paylod_type: PhantomData<T>,
    follow_id: String,
    headers: HashMap<String, String>,
    inner: Rc<dyn MessageInterface>,
    payload: Vec<u8>,
    stream_id: &'static str,
    pub(crate) notify_message_acked: Rc<RefCell<bool>>,
}

impl<T> Clone for Message<T>
where
    T: DeserializeOwned + 'static,
{
    fn clone(&self) -> Message<T> {
        Message {
            _enfoce_paylod_type: PhantomData,
            follow_id: self.follow_id.clone(),
            headers: self.headers.clone(),
            inner: self.inner.clone(),
            notify_message_acked: self.notify_message_acked.clone(),
            payload: self.payload.clone(),
            stream_id: self.stream_id,
        }
    }
}

impl<T> Message<T>
where
    T: DeserializeOwned + 'static,
{
    pub(crate) fn with_backend(
        stream_id: &'static str,
        follow_id: String,
        headers: HashMap<String, String>,
        payload: Vec<u8>,
        inner: Rc<dyn MessageInterface>,
    ) -> Message<T> {
        Message {
            _enfoce_paylod_type: PhantomData,
            follow_id,
            headers,
            inner,
            notify_message_acked: Rc::new(RefCell::new(false)),
            payload,
            stream_id,
        }
    }

    /// Acknowledge that the message was processed and we are done with it.
    ///
    /// The acknowledgement may be asynchronous, with the configured streaming
    /// platform client choosing when to send off the message acknowledgement.
    pub fn async_ack(self) -> Result<()> {
        ACK_TOTAL
            .with_label_values(&[self.stream_id, &self.follow_id])
            .inc();
        *self.notify_message_acked.borrow_mut() = true;
        self.inner.async_ack().map_err(|error| {
            ACK_ERROR
                .with_label_values(&[self.stream_id, &self.follow_id])
                .inc();
            error
        })
    }

    /// Return a streaming platform dependent, message-unique, ID.
    pub fn id(&self) -> &str {
        self.inner.id()
    }

    /// Extract the payload from this message.
    pub fn payload(&self) -> Result<T> {
        serde_json::from_slice(&self.payload)
            .with_context(|_| ErrorKind::PayloadDecode)
            .map_err(Error::from)
    }

    /// Request re-delivery of a message that failed processing.
    pub fn retry(self) {
        // This method is an alias for `drop` to allow code symmetry
        // with task-like interfaces and the tasks system.
        drop(self)
    }

    /// Extract a span context from the message, if present.
    ///
    /// The extracted span context can be used by consumers to
    /// trace the larger flows across processes/systems.
    pub fn trace(&self, tracer: &Tracer) -> OTResult<Option<SpanContext>> {
        let format = ExtractFormat::HttpHeaders(Box::new(&self.headers));
        tracer.extract(format)
    }
}
