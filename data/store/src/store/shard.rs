use replicante_data_models::Shard as ShardModel;

use super::super::backend::ShardImpl;
use super::super::Result;

/// Operate on the shard identified by the provided cluster_id, node_id, shard_id.
pub struct Shard {
    shard: ShardImpl,
    attrs: ShardAttribures,
}

impl Shard {
    pub(crate) fn new(shard: ShardImpl, attrs: ShardAttribures) -> Shard {
        Shard { shard, attrs }
    }

    /// Query the `Node` record, if any is stored.
    pub fn get(&self) -> Result<Option<ShardModel>> {
        self.shard.get(&self.attrs)
    }
}

/// Attributes attached to all shard operations.
pub struct ShardAttribures {
    pub cluster_id: String,
    pub node_id: String,
    pub shard_id: String,
}
