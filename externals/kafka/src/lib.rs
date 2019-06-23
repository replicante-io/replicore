use prometheus::Registry;
use slog::Logger;

mod config;
mod metrics;
mod stats;

pub use self::config::AckLevel;
pub use self::config::CommonConfig;
pub use self::config::Timeouts;
pub use self::stats::ClientStatsContext;
pub use self::stats::KafkaHealthChecker;

/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    self::metrics::register_metrics(logger, registry);
}
