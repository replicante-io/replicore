use replicante_models_agent::info::ShardRole;

#[test]
fn shard_error() {
    let mut report = super::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, _fixture) = super::fixtures::cluster(&mut report);
    let client = super::fixtures::mock_client_err();

    let result = crate::sync::shards::sync_shards(&data, &mut data_mut, &client, "node0");
    assert!(result.is_err());
}

#[test]
fn shard_existing_record() {
    let mut report = super::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, fixture) = super::fixtures::cluster(&mut report);
    let client = super::fixtures::mock_client_ok();

    let result = crate::sync::shards::sync_shards(&data, &mut data_mut, &client, "node0");
    assert!(result.is_ok());

    let key1 = (
        super::fixtures::CLUSTER_ID.into(),
        "node0".into(),
        "shard0".into(),
    );
    let key2 = (
        super::fixtures::CLUSTER_ID.into(),
        "node0".into(),
        "shard1".into(),
    );
    let state = fixture
        .mock_store
        .state
        .lock()
        .expect("MockState lock poisoned");
    let shard1 = state
        .shards
        .get(&key1)
        .cloned()
        .expect("shard1 should be in the store");
    assert_eq!(shard1.role, ShardRole::Secondary);
    let shard2 = state
        .shards
        .get(&key2)
        .cloned()
        .expect("shard2 should be in the store");
    assert_eq!(shard2.role, ShardRole::Unknown("test".into()));
}

#[test]
fn shard_new_record() {
    let mut report = super::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, fixture) = super::fixtures::cluster(&mut report);
    let client = super::fixtures::mock_client_ok();

    let result = crate::sync::shards::sync_shards(&data, &mut data_mut, &client, "node5");
    assert!(result.is_ok());

    let key1 = (
        super::fixtures::CLUSTER_ID.into(),
        "node5".into(),
        "shard0".into(),
    );
    let key2 = (
        super::fixtures::CLUSTER_ID.into(),
        "node5".into(),
        "shard1".into(),
    );
    let state = fixture
        .mock_store
        .state
        .lock()
        .expect("MockState lock poisoned");
    let shard1 = state
        .shards
        .get(&key1)
        .cloned()
        .expect("shard1 should be in the store");
    assert_eq!(shard1.role, ShardRole::Secondary);
    let shard2 = state
        .shards
        .get(&key2)
        .cloned()
        .expect("shard2 should be in the store");
    assert_eq!(shard2.role, ShardRole::Unknown("test".into()));
}
