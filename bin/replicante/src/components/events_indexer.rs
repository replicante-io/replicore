use std::ops::DerefMut;
use std::sync::Arc;

use failure::ResultExt;
use humthreads::Builder as ThreadBuilder;
use humthreads::ThreadScope;
use opentracingrust::AutoFinishingSpan;
use opentracingrust::Tracer;
use sentry::protocol::Breadcrumb;
use sentry::protocol::Map;
use slog::debug;
use slog::Logger;

use replicante_models_core::events::DeserializeResult;
use replicante_models_core::events::Event;
use replicante_models_core::events::EventCode;
use replicante_store_view::store::Store;
use replicante_stream::Error;
use replicante_stream_events::Message;
use replicante_stream_events::Stream;
use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;
use replicante_util_tracing::fail_span;
use replicante_util_upkeep::Upkeep;

use super::Component;
use crate::interfaces::Interfaces;
use crate::ErrorKind;
use crate::Result;

const FOLLOW_GROUP: &str = "events:indexer";

/// Follow the events stream to index events in a searchable store.
pub struct EventsIndexer {
    events: Option<Stream>,
    logger: Option<Logger>,
    store: Option<Store>,
    tracer: Option<Arc<Tracer>>,
}

impl EventsIndexer {
    pub fn new(logger: Logger, interfaces: &mut Interfaces) -> EventsIndexer {
        EventsIndexer {
            events: Some(interfaces.streams.events.clone()),
            logger: Some(logger),
            store: Some(interfaces.stores.view.clone()),
            tracer: Some(interfaces.tracing.tracer()),
        }
    }
}

impl Component for EventsIndexer {
    fn run(&mut self, upkeep: &mut Upkeep) -> Result<()> {
        let events = self
            .events
            .take()
            .expect("EventsIndexer::run called twice?");
        let logger = self
            .logger
            .take()
            .expect("EventsIndexer::run called twice?");
        let store = self.store.take().expect("EventsIndexer::run called twice?");
        let tracer = self
            .tracer
            .take()
            .expect("EventsIndexer::run called twice?");
        debug!(logger, "Starting events indexer thread");
        let thread = ThreadBuilder::new("r:c:events_indexer")
            .full_name("replicore:component:events_indexer")
            .spawn(move |scope| {
                let worker = WorkerThread {
                    events,
                    logger: logger.clone(),
                    store,
                    thread: &scope,
                    tracer,
                };
                if let Err(error) = worker.follow_and_index() {
                    capture_fail!(
                        &error,
                        logger,
                        "Events indexer stopped";
                        failure_info(&error),
                    );
                }
            })
            .with_context(|_| ErrorKind::ThreadSpawn("events indexer"))?;
        upkeep.register_thread(thread);
        Ok(())
    }
}

/// Stream indexer used to keep the code readable.
struct WorkerThread<'a> {
    events: Stream,
    logger: Logger,
    store: Store,
    thread: &'a ThreadScope,
    tracer: Arc<Tracer>,
}

impl<'a> WorkerThread<'a> {
    fn follow_and_index(&self) -> Result<()> {
        let iter = self
            .events
            .follow(FOLLOW_GROUP, self.thread)
            .with_context(|_| ErrorKind::EventsStreamFollow(FOLLOW_GROUP))?;
        self.thread.activity("waiting for events");
        for message in iter {
            let message = match message {
                Ok(message) => message,
                Err(error) => {
                    capture_fail!(
                        &error,
                        self.logger,
                        "Received error while indexing events";
                        failure_info(&error),
                    );
                    continue;
                }
            };
            let _activity = self
                .thread
                .scoped_activity(format!("processing message: {}", message.id()));
            let span = match message.trace(&self.tracer) {
                Ok(context) => context.map(|context| {
                    let mut span = self.tracer.span("events.index");
                    span.follows(context);
                    span.auto_finish()
                }),
                Err(error) => {
                    let error = failure::SyncFailure::new(error);
                    capture_fail!(
                        &error,
                        self.logger,
                        "Unable to extract tracing context from message";
                        failure_info(&error),
                    );
                    None
                }
            };
            let event = Stream::deserialize_event(&message);
            match event {
                DeserializeResult::Ok(event) => self.store_event(event, message, span),
                DeserializeResult::Err(error) => self.invalid_event(error, None, message, span),
                DeserializeResult::Unknown(code, error) => {
                    self.invalid_event(error, Some(code), message, span)
                }
            };
        }
        Ok(())
    }

    fn invalid_event(
        &self,
        error: Error,
        code: Option<EventCode>,
        message: Message,
        mut span: Option<AutoFinishingSpan>,
    ) {
        let message_id = message.id().to_string();
        sentry::with_scope(
            |_| (),
            || {
                sentry::add_breadcrumb(Breadcrumb {
                    category: Some("events.indexer".into()),
                    message: Some("Unrecognised event".into()),
                    data: {
                        let mut map = Map::new();
                        map.insert("message.id".into(), message_id.clone().into());
                        if let Some(code) = code.as_ref() {
                            map.insert("event.category".into(), code.category.clone().into());
                            map.insert("event.code".into(), code.event.clone().into());
                        }
                        map
                    },
                    ..Default::default()
                });
                let event_code = code
                    .as_ref()
                    .map(|code| code.event.clone())
                    .unwrap_or_else(|| "<unknown>".to_string());
                capture_fail!(
                    &error,
                    self.logger,
                    "Unrecognised event";
                    "event.code" => event_code,
                    "message.id" => &message_id,
                    failure_info(&error),
                );
            },
        );
        if let Some(span) = span.as_mut().map(DerefMut::deref_mut) {
            span.tag("message.id", message_id.as_str());
            if let Some(code) = code.as_ref() {
                span.tag("event.category", code.category.as_str());
                span.tag("event.code", code.event.as_str());
            }
            fail_span(error, span);
        }
        message.retry();
    }

    fn store_event(&self, event: Event, message: Message, mut span: Option<AutoFinishingSpan>) {
        let message_id = message.id().to_string();
        let result = self
            .store
            .persist()
            .event(event, span.as_ref().map(|span| span.context().clone()));
        if let Err(error) = result {
            capture_fail!(
                &error,
                self.logger,
                "Failed to perist event to store";
                "message_id" => message_id,
                failure_info(&error),
            );
            fail_span(error, span.as_mut().map(DerefMut::deref_mut));
            message.retry();
            return;
        }
        if let Err(error) = message.async_ack() {
            capture_fail!(
                &error,
                self.logger,
                "Failed to acknowledge event stream message";
                "message_id" => message_id,
                failure_info(&error),
            );
            fail_span(error, span.as_mut().map(DerefMut::deref_mut));
        }
    }
}
