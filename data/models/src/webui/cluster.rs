/// WebUI api model for the largest clusters.
pub type TopClusters = Vec<TopClusterItem>;


/// Overview of a cluster for the WebUI "top clusters" list.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct TopClusterItem {
    pub name: String,
    pub kinds: Vec<String>,
    pub nodes: u32,
}

impl TopClusterItem {
    /// Create a new "top clusters" item.
    pub fn new<S1, S2>(name: S1, kind: S2, nodes: u32) -> TopClusterItem
        where S1: Into<String>,
              S2: Into<String>,
    {
        TopClusterItem {
            name: name.into(),
            kinds: vec![kind.into()],
            nodes,
        }
    }
}


#[cfg(test)]
mod tests {
    use serde_json;

    use super::TopClusterItem;
    use super::TopClusters;

    #[test]
    fn from_json() {
        let payload = r#"[{"name":"c1","kinds":["mongo"],"nodes":4},{"name":"c2","kinds":["redis"],"nodes":2}]"#;
        let clusters: TopClusters = serde_json::from_str(payload).unwrap();
        let c1 = TopClusterItem::new("c1", "mongo", 4);
        let c2 = TopClusterItem::new("c2", "redis", 2);
        let expected = vec![c1, c2];
        assert_eq!(clusters, expected);
    }

    #[test]
    fn to_json() {
        let c1 = TopClusterItem::new("c1", "mongo", 4);
        let c2 = TopClusterItem::new("c2", "redis", 2);
        let clusters = vec![c1, c2];
        let payload = serde_json::to_string(&clusters).unwrap();
        let expected = r#"[{"name":"c1","kinds":["mongo"],"nodes":4},{"name":"c2","kinds":["redis"],"nodes":2}]"#;
        assert_eq!(payload, expected);
    }
}
