use anyhow::Context;
use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;

use super::OrchestrateReport;
use super::OrchestrateReportError;
use super::OrchestrateReportOutcome;
use super::SchedChoice;

/// Incrementally build a report by adding information as cluster orchestration progresses.
pub struct OrchestrateReportBuilder {
    namespace: Option<String>,
    cluster_id: Option<String>,

    start_time: Option<DateTime<Utc>>,
    outcome: Option<OrchestrateReportOutcome>,

    action_scheduling_choices: Option<SchedChoice>,
    node_actions_lost: u64,
    node_actions_schedule_failed: u64,
    node_actions_scheduled: u64,
    nodes_failed: u64,
    nodes_synced: u64,
}

impl OrchestrateReportBuilder {
    pub fn new() -> OrchestrateReportBuilder {
        OrchestrateReportBuilder {
            namespace: None,
            cluster_id: None,
            start_time: None,
            outcome: None,
            action_scheduling_choices: None,
            node_actions_lost: 0,
            node_actions_schedule_failed: 0,
            node_actions_scheduled: 0,
            nodes_failed: 0,
            nodes_synced: 0,
        }
    }

    /// Attach action scheduling choice information to the report.
    pub fn action_scheduling_choices(&mut self, choices: SchedChoice) -> &mut Self {
        self.action_scheduling_choices = Some(choices);
        self
    }

    /// Consume be builder to finalise a report.
    pub fn build(self) -> Result<OrchestrateReport> {
        let namespace = match self.namespace {
            Some(namespace) => namespace,
            None => anyhow::bail!(OrchestrateReportError::MissingClusterIdentifier),
        };
        let start_time = match self.start_time {
            Some(start_time) => start_time,
            None => anyhow::bail!(OrchestrateReportError::MissingStartTime),
        };
        let outcome = match self.outcome {
            Some(outcome) => outcome,
            None => anyhow::bail!(OrchestrateReportError::MissingOutcome),
        };

        let cluster_id = self
            .cluster_id
            .expect("cluster_id must be set if namespace is set");
        let duration = (Utc::now() - start_time)
            .to_std()
            .context(OrchestrateReportError::InvalidTaskDuration)?;
        Ok(OrchestrateReport {
            namespace,
            cluster_id,
            start_time,
            duration,
            outcome,
            action_scheduling_choices: self.action_scheduling_choices,
            node_actions_lost: self.node_actions_lost,
            node_actions_schedule_failed: self.node_actions_schedule_failed,
            node_actions_scheduled: self.node_actions_scheduled,
            nodes_failed: self.nodes_failed,
            nodes_synced: self.nodes_synced,
        })
    }

    /// Set the cluster identification this report is about.
    pub fn for_cluster<S1, S2>(&mut self, namespace: S1, cluster_id: S2) -> &mut Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        self.namespace = Some(namespace.into());
        self.cluster_id = Some(cluster_id.into());
        self
    }

    /// Increment the number of node actions incomplete in core but no longer reported.
    pub fn node_action_lost(&mut self) -> &mut Self {
        self.node_actions_lost += 1;
        self
    }

    /// Increment the number of node actions schedule attempts that failed.
    pub fn node_action_schedule_failed(&mut self) -> &mut Self {
        self.node_actions_schedule_failed += 1;
        self
    }

    /// Increment the number of node actions scheduled.
    pub fn node_action_scheduled(&mut self) -> &mut Self {
        self.node_actions_scheduled += 1;
        self
    }

    /// Increment the number of nodes that failed to sync.
    pub fn node_failed(&mut self) -> &mut Self {
        self.nodes_failed += 1;
        self
    }

    /// Increment the number of nodes that attempted a sync.
    pub fn node_syncing(&mut self) -> &mut Self {
        self.nodes_synced += 1;
        self
    }

    /// Set the outcome of the orchestrate operation.
    pub fn outcome<O>(&mut self, outcome: O) -> &mut Self
    where
        O: Into<OrchestrateReportOutcome>,
    {
        self.outcome = Some(outcome.into());
        self
    }

    /// Set the start time of the orchestrate operation.
    pub fn start_time(&mut self, start_time: DateTime<Utc>) -> &mut Self {
        self.start_time = Some(start_time);
        self
    }

    /// Set the start time of the orchestrate operation to now.
    pub fn start_now(&mut self) -> &mut Self {
        self.start_time(Utc::now())
    }
}

impl Default for OrchestrateReportBuilder {
    fn default() -> Self {
        Self::new()
    }
}
