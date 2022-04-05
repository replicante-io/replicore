use opentracingrust::SpanContext;

use replicante_models_core::agent::Shard as ShardModel;

use super::super::backend::ShardsImpl;
use super::super::Cursor;
use super::super::Result;

/// Operate on all shards in the cluster identified by cluster_id.
pub struct Shards {
    shards: ShardsImpl,
    attrs: ShardsAttribures,
}

impl Shards {
    pub(crate) fn new(shards: ShardsImpl, attrs: ShardsAttribures) -> Shards {
        Shards { shards, attrs }
    }

    /// Iterate over shards in a cluster.
    pub fn iter<S>(&self, span: S) -> Result<Cursor<ShardModel>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.shards.iter(&self.attrs, span.into())
    }
}

/// Attributes attached to all shards operations.
pub struct ShardsAttribures {
    pub cluster_id: String,
}
