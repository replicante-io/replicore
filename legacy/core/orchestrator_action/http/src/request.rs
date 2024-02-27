use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use reqwest::blocking::Client;
use reqwest::tls::Certificate;

use replicante_models_core::actions::orchestrator::OrchestratorAction;

use crate::args::Args;
use crate::response::ResponseInfo;

const ACTION_USER_AGENT: &str = concat!("core.replicante.io+http/", env!("CARGO_PKG_VERSION"));
const DEFAULT_DURATION: Duration = Duration::from_secs(5);

/// Perform the request against the remote.
pub fn perform(action: &OrchestratorAction, args: &Args) -> Result<ResponseInfo> {
    // Initialise a client.
    let mut client = Client::builder().user_agent(ACTION_USER_AGENT);

    if let Some(ca) = &args.remote.ca {
        let ca =
            Certificate::from_pem(ca.as_bytes()).context(crate::errors::ClientError::InvalidCA)?;
        client = client.add_root_certificate(ca);
    }
    if let Some(timeout) = args.remote.timeout {
        client = client.timeout(timeout.map(Duration::from_secs));
    }

    let client = client.build().context(crate::errors::ClientError::Create)?;

    // Send the request to the remote system.
    let response = client
        .post(&args.remote.url)
        .timeout(DEFAULT_DURATION)
        .json(action)
        .send()
        .context(crate::errors::RemoteError::RequestFailed)?;

    // Collect response information for later processing.
    let status = response.status();
    let text = response
        .text()
        .context(crate::errors::RemoteError::ResponseRead)?;
    Ok(ResponseInfo { status, text })
}
