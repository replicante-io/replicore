use chrono::DateTime;
use chrono::Utc;
use opentracingrust::SpanContext;

use replicante_models_core::events::Event;

use crate::backend::EventsImpl;
use crate::Cursor;
use crate::Result;

/// Filters to apply when iterating over events.
pub struct EventsFilters {
    /// Only return cluster-related events if the cluster ID matches.
    ///
    /// Non-cluster events will still be returned.
    pub cluster_id: Option<String>,

    /// Only return events with a matching event code.
    pub event: Option<String>,

    /// Exclude snapshot events from the result (on by default).
    pub exclude_snapshots: bool,

    /// Exclude events that do not relate to a cluster (off by default).
    pub exclude_system_events: bool,

    /// Scan events starting from the given UTC date and time instead of from the oldest event.
    pub start_from: Option<DateTime<Utc>>,

    /// Scan events up to the given UTC date and time instead of up to the newest event.
    pub stop_at: Option<DateTime<Utc>>,
}

impl EventsFilters {
    /// Return all events, don't skip any.
    pub fn all() -> EventsFilters {
        EventsFilters {
            cluster_id: None,
            event: None,
            exclude_snapshots: false,
            exclude_system_events: false,
            start_from: None,
            stop_at: None,
        }
    }

    /// Return all events except for `SNAPSHOT_*` events.
    pub fn most() -> EventsFilters {
        Self::default()
    }
}

impl Default for EventsFilters {
    fn default() -> EventsFilters {
        EventsFilters {
            cluster_id: None,
            event: None,
            exclude_snapshots: true,
            exclude_system_events: false,
            start_from: None,
            stop_at: None,
        }
    }
}

/// Options to apply when iterating over events.
pub struct EventsOptions {
    /// Max number of events to return.
    pub limit: Option<i64>,

    /// By default events are returned old to new, set to true to reverse the order.
    pub reverse: bool,
}

impl Default for EventsOptions {
    fn default() -> EventsOptions {
        EventsOptions {
            limit: None,
            reverse: false,
        }
    }
}

/// Operate on events.
pub struct Events {
    events: EventsImpl,
}

impl Events {
    pub(crate) fn new(events: EventsImpl) -> Events {
        Events { events }
    }

    /// Query historic events.
    pub fn range<S>(
        &self,
        filters: EventsFilters,
        options: EventsOptions,
        span: S,
    ) -> Result<Cursor<Event>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.events.range(filters, options, span.into())
    }
}
