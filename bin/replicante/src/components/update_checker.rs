use std::collections::BTreeMap;

use failure::ResultExt;
use humthreads::Builder;
use lazy_static::lazy_static;
use semver::Version;
use serde_derive::Deserialize;
use slog::warn;
use slog::Logger;

use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;
use replicante_util_upkeep::Upkeep;

use super::Component;
use crate::metrics::UPDATE_AVAILABLE;
use crate::ErrorKind;
use crate::Result;

const UPDATE_META: &str =
    "https://raw.githubusercontent.com/replicante-io/metadata/master/replicante/core/latest.json";

lazy_static! {
    static ref CURRENT_VERSION: Version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
}

/// Fetch information about the latest available replicante core version.
pub struct UpdateChecker {
    logger: Logger,
}

impl UpdateChecker {
    pub fn new(logger: Logger) -> UpdateChecker {
        UpdateChecker { logger }
    }
}

impl Component for UpdateChecker {
    fn run(&mut self, _: &mut Upkeep) -> Result<()> {
        let logger = self.logger.clone();
        Builder::new("r:c:update_checker")
            .full_name("replicante:component:update_checker")
            .spawn(move |scope| {
                let _activity = scope.scoped_activity("checking for updates");
                let response = reqwest::blocking::get(UPDATE_META);
                let response = response.and_then(|response| response.json::<VersionMeta>());
                let response = match response {
                    Ok(response) => response,
                    Err(error) => {
                        capture_fail!(
                            &error,
                            logger,
                            "Failed to fetch latest version information";
                            failure_info(&error)
                        );
                        return;
                    }
                };
                let latest = match Version::parse(&response.version) {
                    Ok(version) => version,
                    Err(error) => {
                        capture_fail!(
                            &error,
                            logger,
                            "Failed to parse latest version information";
                            failure_info(&error)
                        );
                        return;
                    }
                };
                if *CURRENT_VERSION < latest {
                    UPDATE_AVAILABLE.set(1.0);
                    warn!(
                        logger,
                        "A new version is available";
                        "current" => %*CURRENT_VERSION,
                        "latest" => %latest,
                    );
                    sentry::capture_event(sentry::protocol::Event {
                        level: sentry::Level::Warning,
                        message: Some("A new version is available".into()),
                        extra: {
                            let mut extra = BTreeMap::new();
                            extra.insert("current".into(), CURRENT_VERSION.to_string().into());
                            extra.insert("latest".into(), latest.to_string().into());
                            extra
                        },
                        ..Default::default()
                    });
                }
            })
            .with_context(|_| ErrorKind::ThreadSpawn("update_checker"))?;
        Ok(())
    }
}

/// Version metadata returned by the server.
#[derive(Debug, Deserialize)]
struct VersionMeta {
    version: String,
}
