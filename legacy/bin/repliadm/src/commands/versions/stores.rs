use failure::ResultExt;
use slog::Logger;

use replicante::Config;
use replicante_store_primary::admin::Admin as PrimaryStoreAdmin;
use replicante_store_view::admin::Admin as ViewStoreAdmin;

use super::value_or_error;
use crate::Error;
use crate::ErrorKind;
use crate::Result;

/// Collect version information for the configured primary store.
pub fn primary(config: &Config, logger: &Logger) -> Result<()> {
    let version = PrimaryStoreAdmin::make(config.storage.primary.clone(), logger.clone())
        .with_context(|_| ErrorKind::AdminInit("primary store"))
        .and_then(|store| {
            store
                .version()
                .with_context(|_| ErrorKind::FetchVersion("primary store"))
        })
        .map_err(Error::from)
        .map(|v| format!("{} {}", v.tag, v.version));
    println!(
        "==> Primary Store: {}",
        value_or_error(logger, "primary store", version)
    );
    Ok(())
}

/// Collect version information for the configured view store.
pub fn view(config: &Config, logger: &Logger) -> Result<()> {
    let version = ViewStoreAdmin::make(config.storage.view.clone(), logger.clone())
        .with_context(|_| ErrorKind::AdminInit("view store"))
        .and_then(|store| {
            store
                .version()
                .with_context(|_| ErrorKind::FetchVersion("view store"))
        })
        .map_err(Error::from)
        .map(|v| format!("{} {}", v.tag, v.version));
    println!(
        "==> View Store: {}",
        value_or_error(logger, "view store", version)
    );
    Ok(())
}
