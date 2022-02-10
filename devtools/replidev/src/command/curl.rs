use std::path::Path;

use anyhow::Context;
use structopt::StructOpt;
use tokio::process::Command;

use crate::conf::Conf;

/// Curl related options.
#[derive(Debug, StructOpt)]
pub struct Opt {
    /// Arguments passed to curl.
    #[structopt(name = "ARG")]
    arguments: Vec<String>,
}

pub async fn run(args: Opt, conf: Conf) -> anyhow::Result<i32> {
    let mut curl = Command::new("curl");
    if conf.project.allow_gen_certs() {
        let pki_path = <dyn crate::settings::Paths>::pki(&conf.project);
        let ca_cert = format!("{}/replidev/certs/replidev.crt", pki_path);
        let bundle_path = format!("{}/replidev/bundles/client.pem", pki_path);
        if Path::new(&ca_cert).exists() {
            curl.arg("--cacert").arg(ca_cert);
        }
        if Path::new(&bundle_path).exists() {
            curl.arg("--cert").arg(bundle_path);
        }
    }
    curl.args(args.arguments);
    let status = curl
        .status()
        .await
        .context("Failed to execute curl command")?;
    let status = status.code().unwrap_or(127);
    Ok(status)
}
