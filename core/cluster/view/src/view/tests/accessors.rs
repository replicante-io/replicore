use replicante_models_core::agent::ShardRole;

use super::fixtures;
use crate::ClusterView;

#[test]
fn shard_primary_found() {
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
    let cmyk = view
        .shard_primary("cmyk")
        .expect("primary for cmyk to be one")
        .expect("primary for cmyk to be found");
    let hex = view
        .shard_primary("hex")
        .expect("primary for hex to be one")
        .expect("primary for hex to be found");
    let rgb = view
        .shard_primary("rgb")
        .expect("primary for rgb to be one")
        .expect("primary for rgb to be found");
    assert_eq!(cmyk.shard_id, "cmyk");
    assert_eq!(cmyk.role, ShardRole::Primary);
    assert_eq!(hex.shard_id, "hex");
    assert_eq!(hex.role, ShardRole::Primary);
    assert_eq!(rgb.shard_id, "rgb");
    assert_eq!(rgb.role, ShardRole::Primary);
}

#[test]
fn shard_primary_found_many() {
    let discovery = self::fixtures::cluster_mongodb::discovery();
    let settings = self::fixtures::cluster_mongodb::settings();
    let mut builder =
        ClusterView::builder(settings, discovery).expect("ClusterView builder should be created");

    let mut blue_hex = self::fixtures::cluster_mongodb::blue_node_shard_hex();
    blue_hex.role = ShardRole::Primary;
    builder
        .shard(blue_hex)
        .unwrap()
        .shard(self::fixtures::cluster_mongodb::blue_node_shard_rgb())
        .unwrap()
        .shard(self::fixtures::cluster_mongodb::red_node_shard_cmyk())
        .unwrap()
        .shard(self::fixtures::cluster_mongodb::red_node_shard_hex())
        .unwrap();

    let view = builder.build();
    let _cmyk = view
        .shard_primary("cmyk")
        .expect("primary for cmyk to be one")
        .expect("primary for cmyk to be found");
    let _rgb = view
        .shard_primary("rgb")
        .expect("primary for rgb to be one")
        .expect("primary for rgb to be found");
    let hex_error = match view.shard_primary("hex") {
        Err(error) => error,
        _ => panic!("primary for hex should fail"),
    };
    let hex_error: crate::error::ManyPrimariesFound = hex_error
        .downcast()
        .expect("primary for hex should downcast to ManyPrimariesFound");
    assert_eq!(
        hex_error.namespace,
        self::fixtures::cluster_mongodb::NAMESPACE
    );
    assert_eq!(
        hex_error.cluster_id,
        self::fixtures::cluster_mongodb::CLUSTER_ID
    );
    assert_eq!(hex_error.shard_id, "hex");
    assert_eq!(hex_error.records.len(), 2);
}

#[test]
fn shard_primary_not_found() {
    let discovery = self::fixtures::cluster_mongodb::discovery();
    let settings = self::fixtures::cluster_mongodb::settings();
    let mut builder =
        ClusterView::builder(settings, discovery).expect("ClusterView builder should be created");

    builder
        .shard(self::fixtures::cluster_mongodb::blue_node_shard_hex())
        .unwrap()
        .shard(self::fixtures::cluster_mongodb::green_node_shard_rgb())
        .unwrap();

    let view = builder.build();
    let cmyk = view
        .shard_primary("cmyk")
        .expect("primary for cmyk to be one");
    let hex = view
        .shard_primary("hex")
        .expect("primary for hex to be one");
    let rgb = view
        .shard_primary("rgb")
        .expect("primary for rgb to be one");
    assert_eq!(cmyk, None);
    assert_eq!(hex, None);
    assert_eq!(rgb, None);
}
