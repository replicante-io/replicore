use chrono::DateTime;
use chrono::Utc;

use replicante_data_models::ClusterMeta;
use replicante_data_models::Event;

use super::super::backend::LegacyImpl;
use super::super::Cursor;
use super::super::Result;

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

/// Legacy operations that need to be moved to other crates.
pub struct Legacy {
    legacy: LegacyImpl,
}

impl Legacy {
    pub(crate) fn new(legacy: LegacyImpl) -> Legacy {
        Legacy { legacy }
    }

    /// Query a `ClusterMeta` record, if any is stored.
    pub fn cluster_meta(&self, cluster_id: String) -> Result<Option<ClusterMeta>> {
        self.legacy.cluster_meta(cluster_id)
    }

    /// Query historic events.
    pub fn events(&self, filters: EventsFilters, options: EventsOptions) -> Result<Cursor<Event>> {
        self.legacy.events(filters, options)
    }

    /// Query cluster metadata for cluster matching a search term.
    pub fn find_clusters(&self, search: String, limit: u8) -> Result<Cursor<ClusterMeta>> {
        self.legacy.find_clusters(search, limit)
    }

    /// Create or update a ClusterMeta record.
    pub fn persist_cluster_meta(&self, meta: ClusterMeta) -> Result<()> {
        self.legacy.persist_cluster_meta(meta)
    }

    /// Create or update an Event record.
    pub fn persist_event(&self, event: Event) -> Result<()> {
        self.legacy.persist_event(event)
    }

    /// Return the "top cluster" for the WebUI view.
    pub fn top_clusters(&self) -> Result<Cursor<ClusterMeta>> {
        self.legacy.top_clusters()
    }
}
