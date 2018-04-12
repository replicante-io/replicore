/// Cluster metadata shown in the WebUI cluster list.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct ClusterListItem {
    pub name: String,
    pub kinds: Vec<String>,
    pub nodes: u32,
}

impl ClusterListItem {
    /// Create a new "top clusters" item.
    pub fn new<S1, S2>(name: S1, kind: S2, nodes: u32) -> ClusterListItem
        where S1: Into<String>,
              S2: Into<String>,
    {
        ClusterListItem {
            name: name.into(),
            kinds: vec![kind.into()],
            nodes,
        }
    }
}


#[cfg(test)]
mod tests {
    use serde_json;

    use super::ClusterListItem;

    #[test]
    fn from_json() {
        let payload = r#"[{"name":"c1","kinds":["mongo"],"nodes":4},{"name":"c2","kinds":["redis"],"nodes":2}]"#;
        let clusters: Vec<ClusterListItem> = serde_json::from_str(payload).unwrap();
        let c1 = ClusterListItem::new("c1", "mongo", 4);
        let c2 = ClusterListItem::new("c2", "redis", 2);
        let expected = vec![c1, c2];
        assert_eq!(clusters, expected);
    }

    #[test]
    fn to_json() {
        let c1 = ClusterListItem::new("c1", "mongo", 4);
        let c2 = ClusterListItem::new("c2", "redis", 2);
        let clusters = vec![c1, c2];
        let payload = serde_json::to_string(&clusters).unwrap();
        let expected = r#"[{"name":"c1","kinds":["mongo"],"nodes":4},{"name":"c2","kinds":["redis"],"nodes":2}]"#;
        assert_eq!(payload, expected);
    }
}
