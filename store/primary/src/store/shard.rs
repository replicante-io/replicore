use opentracingrust::SpanContext;

use replicante_models_core::agent::Shard as ShardModel;

use super::super::backend::ShardImpl;
use super::super::Result;

/// Operate on the shard identified by the provided cluster_id, node_id, shard_id.
pub struct Shard {
    shard: ShardImpl,
    attrs: ShardAttributes,
}

impl Shard {
    pub(crate) fn new(shard: ShardImpl, attrs: ShardAttributes) -> Shard {
        Shard { shard, attrs }
    }

    /// Query the `Shard` record, if any is stored.
    pub fn get<S>(&self, span: S) -> Result<Option<ShardModel>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.shard.get(&self.attrs, span.into())
    }
}

/// Attributes attached to all shard operations.
pub struct ShardAttributes {
    pub cluster_id: String,
    pub node_id: String,
    pub shard_id: String,
}
