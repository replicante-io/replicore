use humthreads::ThreadScope;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::EmitMessage;
use crate::Iter;
use crate::Result;

/// Internal trait defining backend behaviours.
pub trait MessageInterface {
    /// Inform the streaming plarform that the message was processed.
    ///
    /// The streaming platform should not dispatch this message again to
    /// any clients in the followers group.
    ///
    /// Also see `Message::async_ack` for more details.
    fn async_ack(&self) -> Result<()>;

    /// Return a streaming platform dependent, message-unique, ID.
    fn id(&self) -> &str;
}

/// Internal trait defining backend behaviours.
pub trait StreamInterface<T>: Send + Sync
where
    T: DeserializeOwned + Serialize + 'static,
{
    /// Emit an event to the stream and waits for it to be acknowledged.
    ///
    /// If the backend supports configurable acknowledgements this method
    /// should respect that definition of acknowledgement.
    ///
    /// Also see `Stream::emit` for more details.
    fn emit(&self, message: EmitMessage) -> Result<()>;

    /// Return an `Iterator` that will fetch and return messages from the stream.
    ///
    /// The `group` parameter should be used by the backend to balance partitions
    /// and persist the latest commited offset.
    ///
    /// Also see `Stream::follow` for more details.
    fn follow<'a>(&self, group: String, thread: &'a ThreadScope, tail: bool)
        -> Result<Iter<'a, T>>;
}
