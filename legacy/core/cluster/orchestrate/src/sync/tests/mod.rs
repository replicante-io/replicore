use replisdk::platform::models::ClusterDiscoveryNode;

mod actions;
mod fixtures;
mod info;
mod shards;

use crate::errors::OperationError;

#[test]
fn all_nodes_processed() {
    let mut report = crate::tests::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, _fixture) = self::fixtures::cluster(&mut report);

    super::sync_cluster(&data, &mut data_mut).expect("cluster sync should work");
    report.outcome(&Ok(()));

    let report = report.build().expect("orchestrate report to build");
    assert_eq!(report.nodes_synced, 4);
}

#[test]
fn lock_lost_exits_early() {
    let mut report = crate::tests::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, mut fixture) = self::fixtures::cluster(&mut report);
    fixture
        .lock
        .release(None)
        .expect("fake lock to be released");

    let node = ClusterDiscoveryNode {
        agent_address: "node".into(),
        node_id: "node-test".into(),
    };
    let result = super::sync_node(&data, &mut data_mut, &node);
    match result {
        Ok(()) => panic!("node sync should fail"),
        Err(error) if !error.is::<OperationError>() => panic!("unexpected error from node sync"),
        Err(error) => match error.downcast::<OperationError>().unwrap() {
            OperationError::LockLost(ns, cid) => {
                assert_eq!(ns, crate::tests::fixtures::NAMESPACE);
                assert_eq!(cid, crate::tests::fixtures::CLUSTER_ID);
            }
        },
    }
}
