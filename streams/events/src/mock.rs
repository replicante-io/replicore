use std::sync::Arc;

use replicante_data_models::Event;

use super::EventsStream;
use super::Result;
use super::interface::StreamInterface;


/// Mock implementation of the events streaming interface.
pub struct MockEvents {}

impl MockEvents {
    pub fn new() -> MockEvents {
        MockEvents {}
    }

    pub fn mock(mock: Arc<MockEvents>) -> EventsStream {
        EventsStream(mock)
    }
}

impl StreamInterface for MockEvents {
    fn emit(&self, _event: Event) -> Result<()> {
        Err("Not yet implemented".into())
    }
}
