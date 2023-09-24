//! Container for the complete process runtime configuration.
use serde::Deserialize;
use serde::Serialize;

use replisdk::runtime::shutdown::DEFAULT_SHUTDOWN_GRACE_TIMEOUT;
use replisdk::runtime::tokio_conf::TokioRuntimeConf;

/// Container for the complete process runtime configuration.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeConf {
    /// Allowed time, in seconds, for running operations to complete once process shutdown begins.
    #[serde(default = "RuntimeConf::default_shutdown_grace")]
    pub shutdown_grace_sec: u64,

    /// Tokio Runtime configuration.
    #[serde(default, flatten)]
    pub tokio: TokioRuntimeConf,
}

impl RuntimeConf {
    fn default_shutdown_grace() -> u64 {
        DEFAULT_SHUTDOWN_GRACE_TIMEOUT
    }
}

impl Default for RuntimeConf {
    fn default() -> Self {
        RuntimeConf {
            shutdown_grace_sec: Self::default_shutdown_grace(),
            tokio: Default::default(),
        }
    }
}
