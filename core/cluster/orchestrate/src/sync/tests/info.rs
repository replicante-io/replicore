use replicante_models_core::agent::AgentStatus;

use super::fixtures::mock_client_err;
use super::fixtures::mock_client_ok;
use super::fixtures::mock_client_with_node;

#[test]
fn agent_info_error() {
    let mut report = super::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, _fixture) = super::fixtures::cluster(&mut report);
    let client = mock_client_err();

    let result = crate::sync::info::sync_agent_info(&data, &mut data_mut, &client, "test");
    assert!(result.is_err());
}

#[test]
fn agent_info_existing_record() {
    let mut report = super::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, fixture) = super::fixtures::cluster(&mut report);
    let client = mock_client_ok();

    let result = crate::sync::info::sync_agent_info(&data, &mut data_mut, &client, "node0");
    assert!(result.is_ok());

    let key = (super::fixtures::CLUSTER_ID.into(), "node0".into());
    let info = fixture
        .mock_store
        .state
        .lock()
        .expect("MockState lock poisoned")
        .agents_info
        .get(&key)
        .cloned()
        .expect("the new node to be in the store");
    assert_eq!(info.version_checkout, "abc");
    assert_eq!(info.version_number, "1.2.3");
    assert_eq!(info.version_taint, "all the taints");
}

#[test]
fn agent_info_new_record() {
    let mut report = super::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, fixture) = super::fixtures::cluster(&mut report);
    let client = mock_client_ok();

    let result = crate::sync::info::sync_agent_info(&data, &mut data_mut, &client, "node5");
    assert!(result.is_ok());

    let key = (super::fixtures::CLUSTER_ID.into(), "node5".into());
    let info = fixture
        .mock_store
        .state
        .lock()
        .expect("MockState lock poisoned")
        .agents_info
        .get(&key)
        .cloned()
        .expect("the new node to be in the store");
    assert_eq!(info.version_checkout, "abc");
    assert_eq!(info.version_number, "1.2.3");
    assert_eq!(info.version_taint, "all the taints");
}

#[test]
fn agent_existing_record() {
    let mut report = super::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, fixture) = super::fixtures::cluster(&mut report);

    let result = crate::sync::info::sync_agent(&data, &mut data_mut, "node0", Ok(()), Ok(()));
    assert!(result.is_ok());

    let key = (super::fixtures::CLUSTER_ID.into(), "node0".into());
    let info = fixture
        .mock_store
        .state
        .lock()
        .expect("MockState lock poisoned")
        .agents
        .get(&key)
        .cloned()
        .expect("the new node to be in the store");
    assert_eq!(info.status, AgentStatus::Up);
}

#[test]
fn agent_new_record() {
    let mut report = super::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, fixture) = super::fixtures::cluster(&mut report);

    let result = crate::sync::info::sync_agent(&data, &mut data_mut, "node5", Ok(()), Ok(()));
    assert!(result.is_ok());

    let key = (super::fixtures::CLUSTER_ID.into(), "node5".into());
    let info = fixture
        .mock_store
        .state
        .lock()
        .expect("MockState lock poisoned")
        .agents
        .get(&key)
        .cloned()
        .expect("the new node to be in the store");
    assert_eq!(info.status, AgentStatus::Up);
}

#[test]
fn node_info_error() {
    let mut report = super::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, _fixture) = super::fixtures::cluster(&mut report);
    let client = mock_client_err();

    let result = crate::sync::info::sync_node_info(&data, &mut data_mut, &client, "test");
    assert!(result.is_err());
}

#[test]
fn node_info_existing_record() {
    let mut report = super::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, fixture) = super::fixtures::cluster(&mut report);
    let client = mock_client_ok();

    let result = crate::sync::info::sync_node_info(&data, &mut data_mut, &client, "node0");
    assert!(result.is_ok());

    let key = (super::fixtures::CLUSTER_ID.into(), "node0".into());
    let info = fixture
        .mock_store
        .state
        .lock()
        .expect("MockState lock poisoned")
        .nodes
        .get(&key)
        .cloned()
        .expect("the new node to be in the store");
    assert_eq!(info.kind, "test.db");
    assert_eq!(info.version, "1.2.3");
}

#[test]
fn node_info_new_record() {
    let mut report = super::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, fixture) = super::fixtures::cluster(&mut report);
    let client = mock_client_with_node("node5");

    let result = crate::sync::info::sync_node_info(&data, &mut data_mut, &client, "node5");
    assert!(result.is_ok());

    let key = (super::fixtures::CLUSTER_ID.into(), "node5".into());
    let info = fixture
        .mock_store
        .state
        .lock()
        .expect("MockState lock poisoned")
        .nodes
        .get(&key)
        .cloned()
        .expect("the new node to be in the store");
    assert_eq!(info.kind, "test.db");
    assert_eq!(info.version, "1.2.3");
}

#[test]
fn sync_agent_with_agent_error() {
    let mut report = super::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, fixture) = super::fixtures::cluster(&mut report);
    let client = mock_client_err();

    let agent_info = crate::sync::info::sync_agent_info(&data, &mut data_mut, &client, "node5");
    let node_info = Ok(());
    let result =
        crate::sync::info::sync_agent(&data, &mut data_mut, "node5", agent_info, node_info);
    assert!(result.is_err());

    let key = (super::fixtures::CLUSTER_ID.into(), "node5".into());
    let info = fixture
        .mock_store
        .state
        .lock()
        .expect("MockState lock poisoned")
        .agents
        .get(&key)
        .cloned()
        .expect("the node to be in the store");
    let error = "invalid agent-info response from node node5 in cluster default.colours".into();
    assert_eq!(info.status, AgentStatus::AgentDown(error));
}

#[test]
fn sync_agent_with_node_error() {
    let mut report = super::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, fixture) = super::fixtures::cluster(&mut report);
    let client = mock_client_err();

    let agent_info = Ok(());
    let node_info = crate::sync::info::sync_node_info(&data, &mut data_mut, &client, "node5");
    let result =
        crate::sync::info::sync_agent(&data, &mut data_mut, "node5", agent_info, node_info);
    assert!(result.is_err());

    let key = (super::fixtures::CLUSTER_ID.into(), "node5".into());
    let info = fixture
        .mock_store
        .state
        .lock()
        .expect("MockState lock poisoned")
        .agents
        .get(&key)
        .cloned()
        .expect("the node to be in the store");
    let error = "invalid datastore-info response from node node5 in cluster default.colours".into();
    assert_eq!(info.status, AgentStatus::NodeDown(error));
}

#[test]
fn sync_agent_with_valid_info() {
    let mut report = super::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, fixture) = super::fixtures::cluster(&mut report);

    let agent_info = Ok(());
    let node_info = Ok(());
    let result =
        crate::sync::info::sync_agent(&data, &mut data_mut, "node5", agent_info, node_info);
    assert!(result.is_ok());

    let key = (super::fixtures::CLUSTER_ID.into(), "node5".into());
    let info = fixture
        .mock_store
        .state
        .lock()
        .expect("MockState lock poisoned")
        .agents
        .get(&key)
        .cloned()
        .expect("the node to be in the store");
    assert_eq!(info.status, AgentStatus::Up);
}
