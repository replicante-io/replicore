//! Container for the Control Plane exclusive tasks configuration.
use serde::Deserialize;
use serde::Serialize;

/// Control Plane exclusive tasks.
///
/// Exclusive tasks are executed only by one of the nodes configured to execute them.
/// They are primarily lightweight scheduling and maintenance tasks, and selecting a single node
/// to run them makes the overall system simpler to reason about.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExclusivesConf {
    /// The process is a candidate to run exclusive tasks.
    #[serde(default = "ExclusivesConf::default_candidate")]
    pub candidate: bool,

    /// Intervals, in second, at which to run maintenance tasks.
    #[serde(default)]
    pub maintenance: MaintenanceIntervals,

    /// Intervals, in second, at which to run scheduling tasks.
    #[serde(default)]
    pub schedulers: SchedulerIntervals,
}

impl Default for ExclusivesConf {
    fn default() -> Self {
        Self {
            candidate: Self::default_candidate(),
            maintenance: Default::default(),
            schedulers: Default::default(),
        }
    }
}

impl ExclusivesConf {
    fn default_candidate() -> bool {
        true
    }
}

/// Intervals, in second, at which maintenance tasks should run.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaintenanceIntervals {
    /// Interval, in second, at which to run distributed coordinator backend maintenance.
    #[serde(default = "MaintenanceIntervals::default_coordinator")]
    pub coordinator: u64,
}

impl Default for MaintenanceIntervals {
    fn default() -> Self {
        Self {
            coordinator: Self::default_coordinator(),
        }
    }
}

impl MaintenanceIntervals {
    fn default_coordinator() -> u64 {
        300
    }
}

/// Intervals, in second, at which to run scheduling tasks.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SchedulerIntervals {
    /// Interval, in second, at which to check for cluster orchestrations to schedule.
    #[serde(default = "SchedulerIntervals::default_cluster_orchestrator")]
    pub cluster_orchestrator: u64,

    /// Interval, in second, at which to check for platform discoveries to schedule.
    #[serde(default = "SchedulerIntervals::default_platform_discovery")]
    pub platform_discovery: u64,
}

impl Default for SchedulerIntervals {
    fn default() -> Self {
        Self {
            cluster_orchestrator: Self::default_cluster_orchestrator(),
            platform_discovery: Self::default_platform_discovery(),
        }
    }
}

impl SchedulerIntervals {
    fn default_cluster_orchestrator() -> u64 {
        15
    }

    fn default_platform_discovery() -> u64 {
        15
    }
}
