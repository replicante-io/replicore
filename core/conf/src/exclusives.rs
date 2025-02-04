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
}

impl Default for ExclusivesConf {
    fn default() -> Self {
        Self {
            candidate: Self::default_candidate(),
            maintenance: Default::default(),
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
