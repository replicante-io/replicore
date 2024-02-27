use super::fixtures;
use crate::ClusterView;

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
fn shards_indexed_by_id_and_node() {
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
    let nodes = view
        .shards_by_id
        .get("rgb")
        .expect("shard not found in view");
    assert_eq!(nodes.len(), 2);
}
