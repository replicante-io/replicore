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
        let store_extras: HashMap<&_, &_> = self
            .store_extras
            .iter()
            .map(|(id, extras)| (id, extras.as_ref()))
            .collect();

        state.serialize_field("discovery", &self.discovery)?;
        state.serialize_field("nodes", &nodes)?;
        state.serialize_field("oactions_unfinished", &oactions_unfinished)?;
        state.serialize_field("spec", &self.spec)?;
        state.serialize_field("store_extras", &store_extras)?;
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
            "nodes": {},
            "oactions_unfinished": [],
            "spec": {
                "active": true,
                "cluster_id": "cluster",
                "declaration": {
                    "active": true,
                    "approval": "granted",
                    "definition": null,
                    "grace_up": 5,
                },
                "interval": 60,
                "ns_id": "test",
                "platform": null,
            },
            "store_extras": {},
        });
        assert_eq!(actual, expected);
    }
}
