use super::fixtures;
use crate::ClusterView;

#[test]
fn serialise_cluster_view() {
    let discovery = self::fixtures::cluster_mongodb::discovery();
    let settings = self::fixtures::cluster_mongodb::settings();
    let mut builder =
        ClusterView::builder(settings, discovery).expect("ClusterView builder should be created");

    // Add agents and agents info.
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

    // Add nodes.
    builder
        .node(self::fixtures::cluster_mongodb::blue_node())
        .unwrap()
        .node(self::fixtures::cluster_mongodb::green_node())
        .unwrap()
        .node(self::fixtures::cluster_mongodb::red_node())
        .unwrap();

    // Add shards.
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

    // Add actions.
    builder
        .action(self::fixtures::cluster_mongodb::blue_node_action_restart())
        .unwrap()
        .action(self::fixtures::cluster_mongodb::blue_node_action_stepdown())
        .unwrap()
        .action(self::fixtures::cluster_mongodb::green_node_action_stepdown())
        .unwrap();

    // Serialise the view to check format.
    let view = builder.build();
    let actual = serde_json::to_string_pretty(&view).expect("ClusterView to serialise");
    assert_eq!(actual, std::include_str!("expected-encoded.json"));
}
