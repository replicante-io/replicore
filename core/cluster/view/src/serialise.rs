//! Serialisation for [`ClusterView`] objects
use std::collections::HashMap;

use serde::ser::Serialize;
use serde::ser::SerializeStruct;
use serde::ser::Serializer;

use super::ClusterView;

impl Serialize for ClusterView {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ClusterView", 3)?;
        let nactions_by_node: HashMap<&_, _> = self
            .nactions_by_node
            .iter()
            .map(|(id, actions)| {
                let actions: Vec<&_> = actions.iter().map(|action| action.as_ref()).collect();
                (id, actions)
            })
            .collect();
        let nodes: HashMap<&_, &_> = self
            .nodes
            .iter()
            .map(|(id, node)| (id, node.as_ref()))
            .collect();
        let oactions_unfinished: Vec<&_> = self
            .oactions_unfinished
            .iter()
            .map(|act| act.as_ref())
            .collect();
        let shards: HashMap<&_, _> = self
            .shards
            .iter()
            .map(|(id, shards)| {
                let shards: HashMap<&_, &_> = shards
                    .iter()
                    .map(|(id, shard)| (id, shard.as_ref()))
                    .collect();
                (id, shards)
            })
            .collect();
        let store_extras: HashMap<&_, &_> = self
            .store_extras
            .iter()
            .map(|(id, extras)| (id, extras.as_ref()))
            .collect();

        state.serialize_field("discovery", &self.discovery)?;
        state.serialize_field("nactions_by_node", &nactions_by_node)?;
        state.serialize_field("nodes", &nodes)?;
        state.serialize_field("oactions_unfinished", &oactions_unfinished)?;
        state.serialize_field("spec", &self.spec)?;
        state.serialize_field("shards", &shards)?;
        state.serialize_field("store_extras", &store_extras)?;
        state.serialize_field("stats_shards_by_node", &self.stats_shards_by_node)?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use replisdk::core::models::cluster::ClusterSpec;

    use crate::ClusterView;

    #[test]
    fn serialise() {
        let spec = ClusterSpec::synthetic("test", "cluster");
        let cluster = ClusterView::builder(spec);

        let cluster = cluster.finish();
        let actual = serde_json::to_value(cluster).unwrap();
        let expected = serde_json::json!({
            "discovery": {
                "cluster_id": "cluster",
                "nodes": [],
                "ns_id": "test",
            },
            "nactions_by_node": {},
            "nodes": {},
            "oactions_unfinished": [],
            "spec": {
                "active": true,
                "cluster_id": "cluster",
                "declaration": {
                    "active": true,
                    "approval": "granted",
                    "definition": null,
                    "expand": {
                        "mode": "Auto",
                        "target_member": null,
                    },
                    "graces": {
                        "expand": 5,
                        "init": 5,
                        "scale_up": 5,
                    },
                    "initialise": {
                        "action_args": null,
                        "mode": "Auto",
                        "search": null,
                    },
                },
                "interval": 60,
                "ns_id": "test",
                "platform": null,
            },
            "shards": {},
            "store_extras": {},
            "stats_shards_by_node": {},
        });
        assert_eq!(actual, expected);
    }
}
