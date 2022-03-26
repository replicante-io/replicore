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

    // Serialise the view to check format.
    let view = builder.build();
    let actual = serde_json::to_string_pretty(&view).expect("ClusterView to serialise");
    assert_eq!(
        actual,
        r#"{
  "cluster_id": "colours",
  "namespace": "default",
  "settings": {
    "cluster_id": "colours",
    "enabled": true,
    "interval": 60,
    "namespace": "default"
  },
  "discovery": {
    "cluster_id": "colours",
    "display_name": null,
    "nodes": []
  },
  "agents": {
    "https://blue.mongo.fixtures:12345/": {
      "cluster_id": "colours",
      "host": "https://blue.mongo.fixtures:12345/",
      "status": {
        "code": "AGENT_DOWN",
        "data": "agent error"
      }
    },
    "https://green.mongo.fixtures:12345/": {
      "cluster_id": "colours",
      "host": "https://green.mongo.fixtures:12345/",
      "status": {
        "code": "UP"
      }
    },
    "https://red.mongo.fixtures:12345/": {
      "cluster_id": "colours",
      "host": "https://red.mongo.fixtures:12345/",
      "status": {
        "code": "NODE_DOWN",
        "data": "node error"
      }
    }
  },
  "agents_info": {
    "https://blue.mongo.fixtures:12345/": {
      "cluster_id": "colours",
      "host": "https://blue.mongo.fixtures:12345/",
      "version_checkout": "",
      "version_number": "1.2.3",
      "version_taint": "not tainted"
    },
    "https://green.mongo.fixtures:12345/": {
      "cluster_id": "colours",
      "host": "https://green.mongo.fixtures:12345/",
      "version_checkout": "",
      "version_number": "3.2.1",
      "version_taint": "tainted"
    },
    "https://red.mongo.fixtures:12345/": {
      "cluster_id": "colours",
      "host": "https://red.mongo.fixtures:12345/",
      "version_checkout": "",
      "version_number": "1.2.3",
      "version_taint": "tainted"
    }
  },
  "nodes": {
    "https://blue.mongo.fixtures:12345/": {
      "cluster_display_name": null,
      "cluster_id": "colours",
      "kind": "mongodb",
      "node_id": "https://blue.mongo.fixtures:12345/",
      "version": "4.5.6"
    },
    "https://green.mongo.fixtures:12345/": {
      "cluster_display_name": null,
      "cluster_id": "colours",
      "kind": "mongodb",
      "node_id": "https://green.mongo.fixtures:12345/",
      "version": "6.5.4"
    },
    "https://red.mongo.fixtures:12345/": {
      "cluster_display_name": null,
      "cluster_id": "colours",
      "kind": "mongodb",
      "node_id": "https://red.mongo.fixtures:12345/",
      "version": "4.5.6"
    }
  },
  "shards": [
    {
      "cluster_id": "colours",
      "commit_offset": null,
      "lag": null,
      "node_id": "https://blue.mongo.fixtures:12345/",
      "role": "secondary",
      "shard_id": "hex"
    },
    {
      "cluster_id": "colours",
      "commit_offset": null,
      "lag": null,
      "node_id": "https://blue.mongo.fixtures:12345/",
      "role": "primary",
      "shard_id": "rgb"
    },
    {
      "cluster_id": "colours",
      "commit_offset": null,
      "lag": null,
      "node_id": "https://green.mongo.fixtures:12345/",
      "role": "secondary",
      "shard_id": "cmyk"
    },
    {
      "cluster_id": "colours",
      "commit_offset": null,
      "lag": null,
      "node_id": "https://green.mongo.fixtures:12345/",
      "role": "secondary",
      "shard_id": "rgb"
    },
    {
      "cluster_id": "colours",
      "commit_offset": null,
      "lag": null,
      "node_id": "https://red.mongo.fixtures:12345/",
      "role": "primary",
      "shard_id": "cmyk"
    },
    {
      "cluster_id": "colours",
      "commit_offset": null,
      "lag": null,
      "node_id": "https://red.mongo.fixtures:12345/",
      "role": "primary",
      "shard_id": "hex"
    }
  ],
  "shards_by_id": {
    "cmyk": {
      "https://green.mongo.fixtures:12345/": {
        "node_id": "https://green.mongo.fixtures:12345/",
        "shard_id": "cmyk"
      },
      "https://red.mongo.fixtures:12345/": {
        "node_id": "https://red.mongo.fixtures:12345/",
        "shard_id": "cmyk"
      }
    },
    "hex": {
      "https://blue.mongo.fixtures:12345/": {
        "node_id": "https://blue.mongo.fixtures:12345/",
        "shard_id": "hex"
      },
      "https://red.mongo.fixtures:12345/": {
        "node_id": "https://red.mongo.fixtures:12345/",
        "shard_id": "hex"
      }
    },
    "rgb": {
      "https://blue.mongo.fixtures:12345/": {
        "node_id": "https://blue.mongo.fixtures:12345/",
        "shard_id": "rgb"
      },
      "https://green.mongo.fixtures:12345/": {
        "node_id": "https://green.mongo.fixtures:12345/",
        "shard_id": "rgb"
      }
    }
  },
  "shards_by_node": {
    "https://blue.mongo.fixtures:12345/": {
      "hex": {
        "node_id": "https://blue.mongo.fixtures:12345/",
        "shard_id": "hex"
      },
      "rgb": {
        "node_id": "https://blue.mongo.fixtures:12345/",
        "shard_id": "rgb"
      }
    },
    "https://green.mongo.fixtures:12345/": {
      "cmyk": {
        "node_id": "https://green.mongo.fixtures:12345/",
        "shard_id": "cmyk"
      },
      "rgb": {
        "node_id": "https://green.mongo.fixtures:12345/",
        "shard_id": "rgb"
      }
    },
    "https://red.mongo.fixtures:12345/": {
      "cmyk": {
        "node_id": "https://red.mongo.fixtures:12345/",
        "shard_id": "cmyk"
      },
      "hex": {
        "node_id": "https://red.mongo.fixtures:12345/",
        "shard_id": "hex"
      }
    }
  }
}"#
    );
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
