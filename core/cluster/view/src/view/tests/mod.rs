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
fn agents_must_be_from_cluster() {
    let discovery = self::fixtures::cluster_mongodb::discovery();
    let settings = self::fixtures::cluster_mongodb::settings();
    let mut builder =
        ClusterView::builder(settings, discovery).expect("ClusterView builder should be created");

    let err = builder
        .agent(self::fixtures::cluster_zookeeper::dog_node_agent())
        .err()
        .expect("invalid agent did not fail")
        .downcast::<ClusterViewCorrupt>()
        .expect("unexpected error type");

    match err {
        ClusterViewCorrupt::ClusterIdClash(namespace, expected, found) => {
            assert_eq!(namespace, self::fixtures::cluster_mongodb::NAMESPACE);
            assert_eq!(expected, self::fixtures::cluster_mongodb::CLUSTER_ID);
            assert_eq!(found, self::fixtures::cluster_zookeeper::CLUSTER_ID);
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
fn agents_info_must_be_from_cluster() {
    let discovery = self::fixtures::cluster_mongodb::discovery();
    let settings = self::fixtures::cluster_mongodb::settings();
    let mut builder =
        ClusterView::builder(settings, discovery).expect("ClusterView builder should be created");

    let err = builder
        .agent_info(self::fixtures::cluster_zookeeper::dog_node_agent_info())
        .err()
        .expect("invalid agent info did not fail")
        .downcast::<ClusterViewCorrupt>()
        .expect("unexpected error type");

    match err {
        ClusterViewCorrupt::ClusterIdClash(namespace, expected, found) => {
            assert_eq!(namespace, self::fixtures::cluster_mongodb::NAMESPACE);
            assert_eq!(expected, self::fixtures::cluster_mongodb::CLUSTER_ID);
            assert_eq!(found, self::fixtures::cluster_zookeeper::CLUSTER_ID);
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
        .agent_info(self::fixtures::cluster_mongodb::blue_node_agent_info())
        .unwrap()
        .agent_info(self::fixtures::cluster_mongodb::green_node_agent_info())
        .unwrap()
        .agent_info(self::fixtures::cluster_mongodb::red_node_agent_info())
        .unwrap();

    let view = builder.build();
    assert_eq!(view.agents_info.len(), 3);
}

#[test]
fn nodes_cannot_be_added_twice() {
    let discovery = self::fixtures::cluster_mongodb::discovery();
    let settings = self::fixtures::cluster_mongodb::settings();
    let mut builder =
        ClusterView::builder(settings, discovery).expect("ClusterView builder should be created");

    let err = builder
        .node(self::fixtures::cluster_mongodb::blue_node())
        .unwrap()
        .node(self::fixtures::cluster_mongodb::blue_node())
        .err()
        .expect("duplicate node did not fail")
        .downcast::<ClusterViewCorrupt>()
        .expect("unexpected error type");

    match err {
        ClusterViewCorrupt::DuplicateNode(namespace, cluster_id, node_id) => {
            assert_eq!(namespace, self::fixtures::cluster_mongodb::NAMESPACE);
            assert_eq!(cluster_id, self::fixtures::cluster_mongodb::CLUSTER_ID);
            assert_eq!(node_id, "https://blue.mongo.fixtures:12345/");
        }
        _ => panic!("unexpected error value"),
    };
}

#[test]
fn nodes_info_must_be_from_cluster() {
    let discovery = self::fixtures::cluster_mongodb::discovery();
    let settings = self::fixtures::cluster_mongodb::settings();
    let mut builder =
        ClusterView::builder(settings, discovery).expect("ClusterView builder should be created");

    let err = builder
        .node(self::fixtures::cluster_zookeeper::dog_node())
        .err()
        .expect("invalid node did not fail")
        .downcast::<ClusterViewCorrupt>()
        .expect("unexpected error type");

    match err {
        ClusterViewCorrupt::ClusterIdClash(namespace, expected, found) => {
            assert_eq!(namespace, self::fixtures::cluster_mongodb::NAMESPACE);
            assert_eq!(expected, self::fixtures::cluster_mongodb::CLUSTER_ID);
            assert_eq!(found, self::fixtures::cluster_zookeeper::CLUSTER_ID);
        }
        _ => panic!("unexpected error value"),
    };
}

#[test]
fn nodes_tracked() {
    let discovery = self::fixtures::cluster_mongodb::discovery();
    let settings = self::fixtures::cluster_mongodb::settings();
    let mut builder =
        ClusterView::builder(settings, discovery).expect("ClusterView builder should be created");

    builder
        .node(self::fixtures::cluster_mongodb::blue_node())
        .unwrap()
        .node(self::fixtures::cluster_mongodb::green_node())
        .unwrap()
        .node(self::fixtures::cluster_mongodb::red_node())
        .unwrap();

    let view = builder.build();
    assert_eq!(view.nodes.len(), 3);
}

#[test]
fn shards_cannot_be_added_twice() {
    let discovery = self::fixtures::cluster_mongodb::discovery();
    let settings = self::fixtures::cluster_mongodb::settings();
    let mut builder =
        ClusterView::builder(settings, discovery).expect("ClusterView builder should be created");

    let err = builder
        .shard(self::fixtures::cluster_mongodb::blue_node_shard_hex())
        .unwrap()
        .shard(self::fixtures::cluster_mongodb::blue_node_shard_hex())
        .err()
        .expect("duplicate shard did not fail")
        .downcast::<ClusterViewCorrupt>()
        .expect("unexpected error type");

    match err {
        ClusterViewCorrupt::DuplicateShard(namespace, cluster_id, node_id, shard_id) => {
            assert_eq!(namespace, self::fixtures::cluster_mongodb::NAMESPACE);
            assert_eq!(cluster_id, self::fixtures::cluster_mongodb::CLUSTER_ID);
            assert_eq!(node_id, "https://blue.mongo.fixtures:12345/");
            assert_eq!(shard_id, "hex");
        }
        _ => panic!("unexpected error value"),
    };
}

#[test]
fn shards_indexed_by_node_and_id() {
    let discovery = self::fixtures::cluster_mongodb::discovery();
    let settings = self::fixtures::cluster_mongodb::settings();
    let mut builder =
        ClusterView::builder(settings, discovery).expect("ClusterView builder should be created");

    builder
        .shard(self::fixtures::cluster_mongodb::blue_node_shard_hex())
        .unwrap()
        .shard(self::fixtures::cluster_mongodb::blue_node_shard_rgb())
        .unwrap()
        .shard(self::fixtures::cluster_mongodb::green_node_shard_cmyk())
        .unwrap()
        .shard(self::fixtures::cluster_mongodb::green_node_shard_rgb())
        .unwrap()
        .shard(self::fixtures::cluster_mongodb::red_node_shard_cmyk())
        .unwrap()
        .shard(self::fixtures::cluster_mongodb::red_node_shard_hex())
        .unwrap();

    let view = builder.build();
    let shard = view
        .shard_on_node("https://blue.mongo.fixtures:12345/", "rgb")
        .expect("shard not found on node");
    assert_eq!(shard.node_id, "https://blue.mongo.fixtures:12345/");
    assert_eq!(shard.shard_id, "rgb");
}

#[test]
fn shards_info_must_be_from_cluster() {
    let discovery = self::fixtures::cluster_mongodb::discovery();
    let settings = self::fixtures::cluster_mongodb::settings();
    let mut builder =
        ClusterView::builder(settings, discovery).expect("ClusterView builder should be created");

    let err = builder
        .shard(self::fixtures::cluster_zookeeper::dog_node_shard_maltese())
        .err()
        .expect("invalid shard did not fail")
        .downcast::<ClusterViewCorrupt>()
        .expect("unexpected error type");

    match err {
        ClusterViewCorrupt::ClusterIdClash(namespace, expected, found) => {
            assert_eq!(namespace, self::fixtures::cluster_mongodb::NAMESPACE);
            assert_eq!(expected, self::fixtures::cluster_mongodb::CLUSTER_ID);
            assert_eq!(found, self::fixtures::cluster_zookeeper::CLUSTER_ID);
        }
        _ => panic!("unexpected error value"),
    };
}

#[test]
fn shards_tracked() {
    let discovery = self::fixtures::cluster_mongodb::discovery();
    let settings = self::fixtures::cluster_mongodb::settings();
    let mut builder =
        ClusterView::builder(settings, discovery).expect("ClusterView builder should be created");

    builder
        .shard(self::fixtures::cluster_mongodb::blue_node_shard_hex())
        .unwrap()
        .shard(self::fixtures::cluster_mongodb::blue_node_shard_rgb())
        .unwrap()
        .shard(self::fixtures::cluster_mongodb::green_node_shard_cmyk())
        .unwrap()
        .shard(self::fixtures::cluster_mongodb::green_node_shard_rgb())
        .unwrap()
        .shard(self::fixtures::cluster_mongodb::red_node_shard_cmyk())
        .unwrap()
        .shard(self::fixtures::cluster_mongodb::red_node_shard_hex())
        .unwrap();

    let view = builder.build();
    assert_eq!(view.shards.len(), 6);
}
