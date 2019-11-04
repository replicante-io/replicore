use clap::ArgMatches;
use failure::ResultExt;
use lazy_static::lazy_static;
use reqwest::Client as ReqwestClient;
use slog::Logger;

use replicante_models_core::api::Version;

use super::value_or_error;
use crate::ErrorKind;
use crate::Result;

const ENDPOINT_VERSION: &str = "/api/unstable/introspect/version";

lazy_static! {
    /// Version details for repliadm.
    static ref VERSION: String = format!(
        "{} [{}; {}]",
        env!("CARGO_PKG_VERSION"),
        env!("GIT_BUILD_HASH"),
        env!("GIT_BUILD_TAINT")
    );
}

/// Report all replicante versions (repliadm, static lib, running cluster).
pub fn versions<'a>(args: &ArgMatches<'a>, logger: &Logger) -> Result<()> {
    println!("==> Local repliadm: {}", *VERSION);

    let command = args.subcommand_matches(super::COMMAND).unwrap();
    let cluster = command.value_of("cluster").unwrap();
    let version = api_version(cluster).map(|version| {
        format!(
            "{} [{}; {}]",
            version.version, version.commit, version.taint,
        )
    });
    println!(
        "==> Replicante Core Cluster: {}",
        value_or_error(logger, "replicante dynamic", version)
    );
    Ok(())
}

/// Fetch the version of the Replicante Core Cluster.
///
/// This assumes that all instances run the same version but generally that should be the case.
fn api_version(cluster: &str) -> Result<Version> {
    let url = cluster.trim_end_matches('/');
    let url = format!("{}/{}", url, ENDPOINT_VERSION);
    let client = ReqwestClient::builder()
        .build()
        .with_context(|_| ErrorKind::HttpClient)?;
    let request = client.get(&url);
    let mut response = request
        .send()
        .with_context(|_| ErrorKind::ReplicanteRequest(ENDPOINT_VERSION))?;
    let version = response
        .json()
        .with_context(|_| ErrorKind::ReplicanteJsonDecode)?;
    Ok(version)
}
