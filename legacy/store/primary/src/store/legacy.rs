use opentracingrust::SpanContext;

use replicante_models_core::cluster::ClusterMeta;

use crate::backend::LegacyImpl;
use crate::Cursor;
use crate::Result;

/// Legacy operations that need to be moved to other crates.
pub struct Legacy {
    legacy: LegacyImpl,
}

impl Legacy {
    pub(crate) fn new(legacy: LegacyImpl) -> Legacy {
        Legacy { legacy }
    }

    /// Query a `ClusterMeta` record, if any is stored.
    pub fn cluster_meta<S>(&self, cluster_id: String, span: S) -> Result<Option<ClusterMeta>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.legacy.cluster_meta(cluster_id, span.into())
    }

    /// Query cluster metadata for cluster matching a search term.
    pub fn find_clusters<S>(
        &self,
        search: String,
        limit: u8,
        span: S,
    ) -> Result<Cursor<ClusterMeta>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.legacy.find_clusters(search, limit, span.into())
    }

    /// Create or update a ClusterMeta record.
    pub fn persist_cluster_meta<S>(&self, meta: ClusterMeta, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.legacy.persist_cluster_meta(meta, span.into())
    }

    /// Return the "top cluster" for the WebUI view.
    pub fn top_clusters<S>(&self, span: S) -> Result<Cursor<ClusterMeta>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.legacy.top_clusters(span.into())
    }
}
