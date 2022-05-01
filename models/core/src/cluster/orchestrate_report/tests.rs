use chrono::DateTime;
use chrono::Utc;

use super::OrchestrateReportBuilder;
use super::OrchestrateReportError;

lazy_static::lazy_static! {
    static ref START_TIME: DateTime<Utc> = Utc::now();
    static ref OK: anyhow::Result<()> = Ok(());
}

#[test]
fn build_basic_report() {
    let mut builder = OrchestrateReportBuilder::new();
    builder
        .for_cluster("test-ns", "test-cluster")
        .outcome(&*OK)
        .start_time(*START_TIME);

    let report = builder.build().expect("report failed to build");
    assert_eq!(report.namespace, "test-ns");
    assert_eq!(report.cluster_id, "test-cluster");
    assert_eq!(report.start_time, *START_TIME);
}

#[test]
fn count_node_actions_scheduled_failed_and_lost() {
    let mut builder = OrchestrateReportBuilder::new();
    builder
        .for_cluster("test-ns", "test-cluster")
        .outcome(&*OK)
        .start_time(*START_TIME)
        .node_action_scheduled()
        .node_action_lost()
        .node_action_scheduled()
        .node_action_scheduled()
        .node_action_lost()
        .node_action_schedule_failed()
        .node_action_schedule_failed()
        .node_action_lost()
        .node_action_lost();

    let report = builder.build().expect("report failed to build");
    assert_eq!(report.node_actions_lost, 4);
    assert_eq!(report.node_actions_schedule_failed, 2);
    assert_eq!(report.node_actions_scheduled, 3);
}

#[test]
fn count_nodes_failed_and_synced() {
    let mut builder = OrchestrateReportBuilder::new();
    builder
        .for_cluster("test-ns", "test-cluster")
        .outcome(&*OK)
        .start_time(*START_TIME)
        .node_syncing()
        .node_syncing()
        .node_syncing()
        .node_failed()
        .node_syncing()
        .node_failed()
        .node_syncing();

    let report = builder.build().expect("report failed to build");
    assert_eq!(report.nodes_failed, 2);
    assert_eq!(report.nodes_synced, 5);
}

#[test]
fn for_cluster_required() {
    let builder = OrchestrateReportBuilder::new();
    let report = builder.build();
    match report {
        Ok(_) => panic!("expected builder to fail"),
        Err(error) => {
            let error: OrchestrateReportError = error
                .downcast()
                .expect("builder returned an unexpected error type");
            match error {
                OrchestrateReportError::MissingClusterIdentifier => (),
                _ => panic!("builder returned the incorrect error tag"),
            };
        }
    }
}

#[test]
fn outcome_fail_anyhow() {
    let mock_error = anyhow::anyhow!("the real (fake) reason!")
        .context("even deeper cause")
        .context("error cause one")
        .context("test error");
    let mut builder = OrchestrateReportBuilder::new();
    builder
        .for_cluster("test-ns", "test-cluster")
        .outcome(&Err(mock_error))
        .start_now();
    let report = builder.build().expect("report failed to build");
    assert!(!report.outcome.success, "expected report to show failure");
    assert_eq!(report.outcome.error, Some("test error".into()));
    assert_eq!(
        report.outcome.error_causes,
        vec![
            "error cause one".to_string(),
            "even deeper cause".to_string(),
            "the real (fake) reason!".to_string(),
        ],
    )
}

#[test]
fn outcome_required() {
    let mut builder = OrchestrateReportBuilder::new();
    builder.for_cluster("test-ns", "test-cluster").start_now();
    let report = builder.build();
    match report {
        Ok(_) => panic!("expected builder to fail"),
        Err(error) => {
            let error: OrchestrateReportError = error
                .downcast()
                .expect("builder returned an unexpected error type");
            match error {
                OrchestrateReportError::MissingOutcome => (),
                _ => panic!("builder returned the incorrect error tag"),
            };
        }
    }
}

#[test]
fn outcome_success_anyhow() {
    let mut builder = OrchestrateReportBuilder::new();
    builder
        .for_cluster("test-ns", "test-cluster")
        .outcome(&*OK)
        .start_now();
    let report = builder.build().expect("report failed to build");
    assert!(report.outcome.success, "expected report to show success");
}

#[test]
fn start_now_sets_start_time_to_now() {
    let mut builder = OrchestrateReportBuilder::new();
    builder
        .for_cluster("test-ns", "test-cluster")
        .outcome(&*OK)
        .start_now();
    builder.build().expect("report failed to build");
}

#[test]
fn start_time_required() {
    let mut builder = OrchestrateReportBuilder::new();
    builder.for_cluster("test-ns", "test-cluster");
    let report = builder.build();
    match report {
        Ok(_) => panic!("expected builder to fail"),
        Err(error) => {
            let error: OrchestrateReportError = error
                .downcast()
                .expect("builder returned an unexpected error type");
            match error {
                OrchestrateReportError::MissingStartTime => (),
                _ => panic!("builder returned the incorrect error tag"),
            };
        }
    }
}
