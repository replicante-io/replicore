use crate::ClusterView;
use crate::ClusterViewCorrupt;

mod fixtures;

#[test]
fn build_cluster_view() {
    let discovery = self::fixtures::cluster_mongodb::discovery();
    let settings = self::fixtures::cluster_mongodb::settings();
    let builder =
        ClusterView::builder(settings, discovery).expect("ClusterView builder should be created");
    let view = builder.build();
    assert_eq!(view.namespace, self::fixtures::cluster_mongodb::NAMESPACE);
    assert_eq!(view.cluster_id, self::fixtures::cluster_mongodb::CLUSTER_ID);
}

#[test]
fn fail_when_starting_with_different_cluster_ids() {
    let discovery = self::fixtures::cluster_mongodb::discovery();
    let settings = self::fixtures::cluster_zookeeper::settings();
    let builder = ClusterView::builder(settings, discovery);
    let err = builder
        .err()
        .expect("different cluster ids did not fail building")
        .downcast::<ClusterViewCorrupt>()
        .expect("unexpected error type");
    match err {
        ClusterViewCorrupt::ClusterIdClash(namespace, expected, found) => {
            assert_eq!(namespace, self::fixtures::cluster_mongodb::NAMESPACE);
            assert_eq!(expected, self::fixtures::cluster_zookeeper::CLUSTER_ID);
            assert_eq!(found, self::fixtures::cluster_mongodb::CLUSTER_ID);
        }
        _ => panic!("unexpected error value"),
    };
}

#[test]
fn agents_cannot_be_added_twice() {
    let discovery = self::fixtures::cluster_mongodb::discovery();
    let settings = self::fixtures::cluster_mongodb::settings();
    let mut builder =
        ClusterView::builder(settings, discovery).expect("ClusterView builder should be created");

    let err = builder
        .agent(self::fixtures::cluster_mongodb::blue_node_agent())
        .unwrap()
        .agent(self::fixtures::cluster_mongodb::blue_node_agent())
        .err()
        .expect("duplicate agent did not fail")
        .downcast::<ClusterViewCorrupt>()
        .expect("unexpected error type");

    match err {
        ClusterViewCorrupt::DuplicateAgent(namespace, cluster_id, agent_id) => {
            assert_eq!(namespace, self::fixtures::cluster_mongodb::NAMESPACE);
            assert_eq!(cluster_id, self::fixtures::cluster_mongodb::CLUSTER_ID);
            assert_eq!(agent_id, "https://blue.mongo.fixtures:12345/");
        }
        _ => panic!("unexpected error value"),
    };
}

#[test]
fn agents_tracked() {
    let discovery = self::fixtures::cluster_mongodb::discovery();
    let settings = self::fixtures::cluster_mongodb::settings();
    let mut builder =
        ClusterView::builder(settings, discovery).expect("ClusterView builder should be created");

    builder
        .agent(self::fixtures::cluster_mongodb::blue_node_agent())
        .unwrap()
        .agent(self::fixtures::cluster_mongodb::green_node_agent())
        .unwrap()
        .agent(self::fixtures::cluster_mongodb::red_node_agent())
        .unwrap();

    let view = builder.build();
    assert_eq!(view.agents.len(), 3);
    assert_eq!(view.agents_info.len(), 0);
}

#[test]
fn agents_info_cannot_be_added_twice() {
    let discovery = self::fixtures::cluster_mongodb::discovery();
    let settings = self::fixtures::cluster_mongodb::settings();
    let mut builder =
        ClusterView::builder(settings, discovery).expect("ClusterView builder should be created");

    let err = builder
        .agent(self::fixtures::cluster_mongodb::blue_node_agent())
        .unwrap()
        .agent_info(self::fixtures::cluster_mongodb::blue_node_agent_info())
        .unwrap()
        .agent_info(self::fixtures::cluster_mongodb::blue_node_agent_info())
        .err()
        .expect("duplicate agent info did not fail")
        .downcast::<ClusterViewCorrupt>()
        .expect("unexpected error type");

    match err {
        ClusterViewCorrupt::DuplicateAgentInfo(namespace, cluster_id, agent_id) => {
            assert_eq!(namespace, self::fixtures::cluster_mongodb::NAMESPACE);
            assert_eq!(cluster_id, self::fixtures::cluster_mongodb::CLUSTER_ID);
            assert_eq!(agent_id, "https://blue.mongo.fixtures:12345/");
        }
        _ => panic!("unexpected error value"),
    };
}

#[test]
fn agents_info_tracked() {
    let discovery = self::fixtures::cluster_mongodb::discovery();
    let settings = self::fixtures::cluster_mongodb::settings();
    let mut builder =
        ClusterView::builder(settings, discovery).expect("ClusterView builder should be created");

    builder
        .agent(self::fixtures::cluster_mongodb::blue_node_agent())
        .unwrap()
        .agent_info(self::fixtures::cluster_mongodb::blue_node_agent_info())
        .unwrap()
        .agent(self::fixtures::cluster_mongodb::green_node_agent())
        .unwrap()
        .agent_info(self::fixtures::cluster_mongodb::green_node_agent_info())
        .unwrap()
        .agent(self::fixtures::cluster_mongodb::red_node_agent())
        .unwrap()
        .agent_info(self::fixtures::cluster_mongodb::red_node_agent_info())
        .unwrap();

    let view = builder.build();
    assert_eq!(view.agents.len(), 3);
    assert_eq!(view.agents_info.len(), 3);
}
