use chrono::DateTime;
use chrono::Utc;
use opentracingrust::SpanContext;

use replicante_data_models::Event;

use super::Result;

/// Private interface to the events streaming system.
///
/// Allows multiple possible backends to be used as well as mocks for testing.
pub trait StreamInterface: Send + Sync {
    /// Emit events to the events stream.
    fn emit(&self, event: Event, span: Option<SpanContext>) -> Result<()>;

    /// Scan for events matching the given filters, old to new.
    fn scan(
        &self,
        filters: ScanFilters,
        options: ScanOptions,
        span: Option<SpanContext>,
    ) -> Result<Iter>;
}

/// Iterator over events returned by a scan operation.
pub struct Iter(Box<dyn Iterator<Item = Result<Event>>>);

impl Iter {
    pub fn new<I>(iter: I) -> Iter
    where
        I: Iterator<Item = Result<Event>> + 'static,
    {
        Iter(Box::new(iter))
    }
}

impl Iterator for Iter {
    type Item = Result<Event>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// Filters to apply when scanning events.
pub struct ScanFilters {
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

impl ScanFilters {
    /// Return all events, don't skip any (include snapshots).
    pub fn all() -> ScanFilters {
        let mut filters = Self::default();
        filters.exclude_snapshots = false;
        filters
    }

    /// Return all events except snapshot events.
    pub fn most() -> ScanFilters {
        Self::default()
    }
}

impl Default for ScanFilters {
    fn default() -> ScanFilters {
        ScanFilters {
            cluster_id: None,
            event: None,
            exclude_snapshots: true,
            exclude_system_events: false,
            start_from: None,
            stop_at: None,
        }
    }
}

/// Options to apply when scanning events.
pub struct ScanOptions {
    /// Max number of events to return.
    pub limit: Option<i64>,

    /// By default events are returned old to new, set to true to reverse the order.
    pub reverse: bool,
}

impl Default for ScanOptions {
    fn default() -> ScanOptions {
        ScanOptions {
            limit: None,
            reverse: false,
        }
    }
}
