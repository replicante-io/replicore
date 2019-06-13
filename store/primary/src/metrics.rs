use prometheus::Registry;
use slog::Logger;

pub fn register_metrics(logger: &Logger, registry: &Registry) {
    self::super::backend::register_metrics(logger, registry);
}
