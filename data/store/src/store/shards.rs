use opentracingrust::SpanContext;
use serde_derive::Deserialize;
use serde_derive::Serialize;

use replicante_data_models::Shard as ShardModel;

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

    /// Count the number of active shards in a cluster.
    ///
    /// Active nodes are those not stale.
    /// See `Store::cluster::mark_stale` for why nodes are marked stale.
    pub fn counts<S>(&self, span: S) -> Result<ShardsCounts>
    where
        S: Into<Option<SpanContext>>,
    {
        self.shards.counts(&self.attrs, span.into())
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

/// Counts returned by the `Shards::counts` operation.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct ShardsCounts {
    pub primaries: i32,
    pub shards: i32,
}
