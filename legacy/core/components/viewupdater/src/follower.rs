use std::sync::Arc;

use failure::ResultExt;
use humthreads::ThreadScope;
use opentracingrust::AutoFinishingSpan;
use opentracingrust::Tracer;
use sentry::protocol::Breadcrumb;
use sentry::protocol::Map;
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

use crate::ErrorKind;
use crate::Result;

const FOLLOW_GROUP: &str = "events:viewupdater";

/// Stream indexer used to keep the code readable.
pub struct Follower<'a> {
    pub events: Stream,
    pub logger: Logger,
    pub store: Store,
    pub thread: &'a ThreadScope,
    pub tracer: Arc<Tracer>,
}

impl<'a> Follower<'a> {
    /// Extract a span from the message, if a context was set.
    fn span_for_message(&self, message: &Message) -> Option<AutoFinishingSpan> {
        match message.trace(&self.tracer) {
            Ok(context) => context.map(|context| {
                let mut span = self.tracer.span("events.viewupdater");
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
        }
    }

    /// Follow the event stream and update the view DB based on received events.
    pub fn update_view_db(&self) -> Result<()> {
        let iter = self
            .events
            .follow(FOLLOW_GROUP, self.thread)
            .context(ErrorKind::EventsStreamFollow)?;
        self.thread.activity("waiting for events");
        for message in iter {
            let message = message.context(ErrorKind::EventsStreamFollow)?;
            let _activity = self
                .thread
                .scoped_activity(format!("processing message: {}", message.id()));
            let span = self.span_for_message(&message);
            let event = Stream::deserialize_event(&message);
            match event {
                DeserializeResult::Ok(event) => self.process(event, message, span)?,
                DeserializeResult::Err(error) => {
                    let message_id = message.id().to_string();
                    self.report_event_error(error, None, &message_id, span);
                    return Err(ErrorKind::EventHasNoCode(message_id).into());
                }
                DeserializeResult::Unknown(code, error) => {
                    let message_id = message.id().to_string();
                    self.report_event_error(error, code, &message_id, span);
                    // Presume an upgrade is ongoing and retry the message.
                    // We don't expect to succeed here but instead preserve other functions on the
                    // node until the presumed upgrade process replaces us.
                    message.retry();
                }
            };
        }
        Ok(())
    }

    fn report_event_error<C>(
        &self,
        error: Error,
        code: C,
        message_id: &str,
        mut span: Option<AutoFinishingSpan>,
    ) where
        C: Into<Option<EventCode>>,
    {
        let code = code.into();
        sentry::with_scope(
            |_| (),
            || {
                sentry::add_breadcrumb(Breadcrumb {
                    category: Some("events.viewupdater".into()),
                    message: Some("Unrecognised event".into()),
                    data: {
                        let mut map = Map::new();
                        map.insert("message.id".into(), message_id.into());
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
                    .map(|code| code.event.as_str())
                    .unwrap_or("<unknown>");
                capture_fail!(
                    &error,
                    self.logger,
                    "Unrecognised event";
                    "event.code" => event_code,
                    "message.id" => message_id,
                    failure_info(&error),
                );
            },
        );
        if let Some(span) = span.as_deref_mut() {
            span.tag("message.id", message_id);
            if let Some(code) = code.as_ref() {
                span.tag("event.category", code.category.as_str());
                span.tag("event.code", code.event.as_str());
            }
            fail_span(error, span);
        }
    }

    fn process(
        &self,
        event: Event,
        message: Message,
        mut span: Option<AutoFinishingSpan>,
    ) -> Result<()> {
        // Persist information based on the received event.
        let message_id = message.id().to_string();
        let result = super::by_event::process(self, &event, span.as_deref_mut());
        if let Err(error) = result {
            capture_fail!(
                &error,
                self.logger,
                "Failed to process event to update the view store";
                "message_id" => &message_id,
                failure_info(&error),
            );
            fail_span(error, span.as_deref_mut());
            message.retry();
            return Ok(());
        }

        // Persist the event to the events index.
        let result = self
            .store
            .persist()
            .event(event, span.as_ref().map(|span| span.context().clone()));
        if let Err(error) = result {
            capture_fail!(
                &error,
                self.logger,
                "Failed to index event to view store";
                "message_id" => &message_id,
                failure_info(&error),
            );
            fail_span(error, span.as_deref_mut());
            message.retry();
            return Ok(());
        }
        message
            .async_ack()
            .map_err(|error| fail_span(error, span.as_deref_mut()))
            .with_context(|_| ErrorKind::EventsStreamAck(message_id))?;
        Ok(())
    }
}
