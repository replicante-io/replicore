use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::RwLock;

use anyhow::Context;
use anyhow::Result;
use failure::Fail;
use serde::Deserialize;

use replisdk::core::models::platform::Platform;
use replisdk::core::models::platform::PlatformTransport;
use replisdk::core::models::platform::PlatformTransportHttp;
use replisdk::platform::models::NodeProvisionRequest;

use replicante_models_core::actions::orchestrator::OrchestratorAction as OARecord;
use replicante_models_core::actions::orchestrator::OrchestratorActionScheduleMode;
use replicante_models_core::actions::orchestrator::OrchestratorActionState;
use replicante_store_primary::store::Store;
use replicore_iface_orchestrator_action::registry_entry_factory;
use replicore_iface_orchestrator_action::OrchestratorAction;
use replicore_iface_orchestrator_action::ProgressChanges;


/// A simple action that return the arguments it was called for as output.
#[derive(Default)]
pub struct Provision {
    // TODO(global-singleton): Drop this state and the set_store hack.
    initialised: AtomicBool,
    store: RwLock<Option<Store>>,
}

registry_entry_factory! {
    handler: Provision,
    schedule_mode: OrchestratorActionScheduleMode::Exclusive,
    summary: "Use a Platform to request provisioning of a new cluster node",
    // This action only requests provisioning so a short timeout is appropriate.
    timeout: std::time::Duration::from_secs(10 * 60),
}

impl Provision {
    /// Access the Store instance tracked by this action handler.
    fn get_store(&self) -> Store {
        self
            .store
            .read()
            .expect("platform.replicante.io/node.provision store lock poisoned")
            .as_ref()
            .expect("platform.replicante.io/node.provision store not initialised")
            .clone()
    }
}

impl OrchestratorAction for Provision {
    fn progress(&self, record: &OARecord) -> Result<Option<ProgressChanges>> {
        // Decode arguments to the action.
        let args: ProvisionArgs = serde_json::from_value(record.args.clone())?;

        // Reject actions provisioning nodes for other clusters
        if record.cluster_id != args.request.cluster.cluster_id {
            anyhow::bail!(crate::errors::Arguments::invalid_cluster_scope(
                &record.cluster_id,
                args.request.cluster.cluster_id,
            ));
        }

        // Lookup the Platform instance to provision nodes with.
        let ns_id = args.platform_ref.namespace.unwrap_or_else(|| "default".into());
        let platform_id = args.platform_ref.name;
        let store = self.get_store();
        let platform = store
            .platform(ns_id, platform_id)
            .get(None)
            .map_err(|error| error.compat())?
            .context(crate::errors::Platform::NotFound)?;
        if !platform.active {
            anyhow::bail!(crate::errors::Platform::NotActive);
        }

        // Submit node provisioning request to the platform.
        let response = platform_provision(platform, args.request)?;
        Ok(Some(ProgressChanges {
            state: OrchestratorActionState::Done,
            state_payload: Some(Some(response)),
            state_payload_error: None,
        }))
    }

    fn set_store(&self, store: &Store) {
        if !self.initialised.load(Ordering::SeqCst) {
            let mut lock = self
                .store
                .write()
                .expect("platform.replicante.io/node.provision store lock poisoned");
            *lock = Some(store.clone());
            self.initialised.store(true, Ordering::SeqCst);
        }
    }
}

/// Reference to the Platform to provision the node with.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
struct PlatformRef {
    /// Namespace to lookup the platform from (defaults to same as action).
    #[serde(default)]
    namespace: Option<String>,

    /// Name of the platform to lookup.
    name: String,
}

/// Arguments to a node `Provision` action.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
struct ProvisionArgs {
    /// Reference the Platform instance to provision nodes with.
    platform_ref: PlatformRef,

    /// Node provisioning request to send to the platform.
    #[serde(flatten)]
    request: NodeProvisionRequest,
}

/// Provision a node using the platform.
fn platform_provision(
    platform: Platform,
    request: NodeProvisionRequest,
) -> Result<serde_json::Value> {
    match platform.transport {
        PlatformTransport::Http(http) => platform_provision_http(http, request),
    }
}

/// Provision a node using the platform over HTTP(S).
fn platform_provision_http(
    transport: PlatformTransportHttp,
    request: NodeProvisionRequest,
) -> Result<serde_json::Value> {
    let mut client = reqwest::blocking::Client::builder();
    if let Some(ca_cert) = transport.tls_ca_bundle {
        let cert = reqwest::Certificate::from_pem(ca_cert.as_bytes())?;
        client = client.add_root_certificate(cert);
    }
    if transport.tls_insecure_skip_verify {
        client = client.danger_accept_invalid_certs(true);
    }

    let client = client.build()?;
    let url = reqwest::Url::parse(&transport.base_url)?.join("/provision")?;
    let response = client
        .post(url)
        .json(&request)
        .send()?;
    match response.error_for_status_ref() {
        Ok(_) => Ok(response.json()?),
        Err(error) => {
            let body = response.text()?;
            let details = anyhow::anyhow!(body)
                .context(error);
            anyhow::bail!(details);
        }
    }
}
