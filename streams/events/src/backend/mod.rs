use std::sync::Arc;
use slog::Logger;

use replicante_data_store::store::Store;

use super::config::Config;
use super::interface::StreamInterface;


mod store;


pub fn new(config: Config, logger: Logger, store: Store) -> Arc<StreamInterface> {
    match config {
        Config::Store => self::store::StoreInterface::new(logger, store),
    }
}
