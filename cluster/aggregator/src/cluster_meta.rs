use std::collections::HashSet;

use failure::ResultExt;
use opentracingrust::Span;

use replicante_data_models::ClusterDiscovery;
use replicante_data_models::ClusterMeta;
use replicante_data_store::store::Store;

use super::ErrorKind;
use super::Result;

pub(crate) struct ClusterMetaAggregator {
    agents_down: i32,
    cluster_display_name: Option<String>,
    cluster_id: String,
    kinds: HashSet<String>,
    nodes: i32,
    nodes_down: i32,
    shards_count: i32,
    shards_primaries: i32,
}

impl ClusterMetaAggregator {
    /// Fetch and aggrgate cluster metadata.
    pub(crate) fn aggregate(&mut self, store: Store, span: &mut Span) -> Result<()> {
        // Fetch nodes counts.
        let counts = store
            .agents(self.cluster_id.clone())
            .counts(span.context().clone())
            .with_context(|_| ErrorKind::StoreRead("agents counts"))?;
        self.agents_down = counts.agents_down;
        self.nodes = counts.nodes;
        self.nodes_down = counts.nodes_down;

        // Fetch known datastore kinds.
        self.kinds = store
            .nodes(self.cluster_id.clone())
            .kinds(span.context().clone())
            .with_context(|_| ErrorKind::StoreRead("nodes kinds"))?;

        // Fetch total shards count.
        let counts = store
            .shards(self.cluster_id.clone())
            .counts(span.context().clone())
            .with_context(|_| ErrorKind::StoreRead("shards counts"))?;
        self.shards_count = counts.shards;
        self.shards_primaries = counts.primaries;
        Ok(())
    }

    /// Convert the generated data into a `ClusterMeta` record.
    pub(crate) fn generate(mut self) -> ClusterMeta {
        if self.cluster_display_name.is_none() {
            // A cluster_display_name is None if no Node was fetched from the cluster
            // (in case all nodes or agents are down).
            // In that case default to the cluster ID for the display name.
            self.cluster_display_name = Some(self.cluster_id.clone());
        }

        // Build the model.
        let cluster_id = self.cluster_id;
        let cluster_display_name = self.cluster_display_name.take().unwrap();
        let mut meta = ClusterMeta::new(cluster_id.clone(), cluster_display_name);
        meta.agents_down = self.agents_down;
        meta.kinds = self.kinds.into_iter().collect();
        meta.nodes = self.nodes;
        meta.nodes_down = self.nodes_down;
        meta.shards_count = self.shards_count;
        meta.shards_primaries = self.shards_primaries;
        meta
    }

    pub(crate) fn new(discovery: &ClusterDiscovery) -> ClusterMetaAggregator {
        ClusterMetaAggregator {
            agents_down: 0,
            cluster_display_name: discovery.display_name.clone(),
            cluster_id: discovery.cluster_id.clone(),
            kinds: HashSet::new(),
            nodes: 0,
            nodes_down: 0,
            shards_count: 0,
            shards_primaries: 0,
        }
    }
}
