//! Request provisioning of a new node from a Platform server and wait for it to become visible.
use std::collections::HashSet;
use std::time::Duration;

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use time::OffsetDateTime;

use replisdk::core::models::oaction::OActionState;
use replisdk::platform::models::NodeProvisionRequest;
use replisdk::platform::models::NodeProvisionRequestDetails;

use replicore_context::Context;
use replicore_injector::Injector;
use replicore_oaction::OActionChanges;
use replicore_oaction::OActionHandler;
use replicore_oaction::OActionInvokeArgs;
use replicore_oaction::OActionMetadata;
use replicore_store::query::LookupPlatform;

/// Request provisioning of a new node from a Platform server and wait for it to become visible.
///
/// This provisioning action makes a single attempt to provision the node from the Platform server.
/// After the provisioning request is made successfully the action waits for a number of new nodes
/// to appear in the group based on the cluster discovery record.
///
/// The same set of new nodes must appear for a minimum number of orchestration cycles
/// before the new nodes are considered provisioned.
/// This avoids node initialisation failures or transient platform issues that prevent
/// correct node creation to appear successful while the node will shortly fail.
/// This approach also allows auto-retry mechanisms the platform may provide to transparently
/// rectify transient issues.
///
/// The provisioning action does not wait for the node to become fully health as
/// that can take some time with cluster joining and replication catchup steps.
#[derive(Debug)]
pub struct ProvisionNodes;


impl ProvisionNodes {
    /// Registration metadata for the `core.replicante.io/platform.provision` action.
    pub fn metadata() -> OActionMetadata {
        let mut builder =
            OActionMetadata::build(format!("{}.provision", crate::KIND_PREFIX), ProvisionNodes);
        builder.timeout(crate::PROVISION_TIMEOUT);
        builder.finish()
    }

    /// First invocation logic that collects needed state and perform the provisioning request.
    async fn invoke_provision(
        &self,
        context: &Context,
        mut state: ProvisionNodesState,
        invoke: &OActionInvokeArgs<'_>,
        args: &ProvisionNodesArgs,
    ) -> Result<ProvisionNodesState> {
        // Ensure required definitions are available.
        let cluster_def = match &invoke.spec.declaration.definition {
            None => anyhow::bail!(
                "ProvisionNodes action requires a cluster definition to be configured"
            ),
            Some(def) => def,
        };
        let platform = match &invoke.spec.platform {
            None => anyhow::bail!("ProvisionNodes action requires a platform to be configured"),
            Some(platform) => platform,
        };

        // Track current nodes in the group so we can tell when new ones show up.
        state.starting_nodes = self.nodes_ids(invoke, &args.request.node_group_id);

        // Ask the platform to provision the requested nodes.
        let injector = Injector::global();
        let lookup = LookupPlatform::by(
            platform.ns_id.as_ref().unwrap_or(&invoke.spec.ns_id),
            &platform.name,
        );
        let platform = match injector.store.query(context, lookup).await? {
            None => anyhow::bail!("ClusterSpec references a non existing platform"),
            Some(platform) => platform,
        };
        let client = injector
            .clients
            .platform
            .factory(context, &platform)
            .await?;
        let response = client
            .provision(NodeProvisionRequest {
                cluster: cluster_def.clone(),
                provision: args.request.clone(),
            })
            .await?;

        state.expected = response.count;
        state.requested = true;
        state.stable = None;
        Ok(state)
    }

    /// Subsequent invocations wait for new nodes to show in the discovery record.
    async fn invoke_wait(
        &self,
        mut state: ProvisionNodesState,
        invoke: &OActionInvokeArgs<'_>,
        args: &ProvisionNodesArgs,
    ) -> Result<ProvisionNodesState> {
        // Find nodes in the group added since the first invocation.
        let current_nodes = self.nodes_ids(invoke, &args.request.node_group_id);
        let added_nodes: HashSet<_> = current_nodes
            .difference(&state.starting_nodes)
            .cloned()
            .collect();

        // Wait for all expected nodes to be added.
        let len = u32::try_from(added_nodes.len()).expect("added nodes count too large");
        if len < state.expected {
            state.stable = None;
            return Ok(state);
        }

        // If the candidates set has changed reset our tracking.
        let same_candidates = state.candidate_nodes == added_nodes;
        if !same_candidates {
            state.candidate_nodes = added_nodes;
            state.stable = Some(OffsetDateTime::now_utc());
        }

        // We should never get to the same candidates but no stable timestamp but just in case.
        if same_candidates && state.stable.is_none() {
            state.stable = Some(OffsetDateTime::now_utc());
        }
        Ok(state)
    }

    /// Grab the set of discovered node IDs for the given group.
    fn nodes_ids(&self, args: &OActionInvokeArgs, node_group_id: &str) -> HashSet<String> {
        args
            .discovery
            .nodes
            .iter()
            .filter(|node| match &node.node_group {
                Some(group) => group == &node_group_id,
                None => false,
            })
            .map(|node| node.node_id.clone())
            .collect()
    }
}

#[async_trait::async_trait]
impl OActionHandler for ProvisionNodes {
    async fn invoke(&self, context: &Context, invoke: &OActionInvokeArgs) -> Result<OActionChanges> {
        let args: ProvisionNodesArgs = serde_json::from_value(invoke.action.args.clone())?;
        let state = match &invoke.action.state_payload {
            None => ProvisionNodesState::default(),
            Some(state) => serde_json::from_value(state.clone())?,
        };

        // Attempt provisioning once, then watch discovery to know when we are done.
        let state = if !state.requested {
            self.invoke_provision(context, state, invoke, &args).await?
        } else {
            self.invoke_wait(state, invoke, &args).await?
        };

        // Determine if the action is done by looking at how long nodes have been stable.
        let stable_after = Duration::from_secs(args.stable_after);
        let now = OffsetDateTime::now_utc();
        let next = match state.stable {
            Some(time) if time + stable_after >= now => OActionState::Done,
            _ => OActionState::Running,
        };
        let state = serde_json::to_value(state)?;
        Ok(OActionChanges::to(next).payload(state))
    }
}

/// Arguments required and supported by the [`ProvisionNodes`] action.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProvisionNodesArgs {
    /// Details of the nodes to provision.
    pub request: NodeProvisionRequestDetails,

    /// Amount of time, in seconds, to wait for the new nodes to be stable before we are done.
    #[serde(default = "ProvisionNodesArgs::default_stable_after")]
    pub stable_after: u64,
}

impl ProvisionNodesArgs {
    /// Default time, in seconds, to wait for the new nodes to be stable before the action is done.
    pub fn default_stable_after() -> u64 {
        5 * 60
    }
}

impl From<NodeProvisionRequestDetails> for ProvisionNodesArgs {
    fn from(value: NodeProvisionRequestDetails) -> Self {
        ProvisionNodesArgs {
            request: value,
            stable_after: ProvisionNodesArgs::default_stable_after(),
        }
    }
}

/// Current state of a [`ProvisionNodes`] action.
#[derive(Serialize, Deserialize)]
struct ProvisionNodesState {
    /// Node IDs for nodes that appeared in the node group since the provisioning request was made.
    candidate_nodes: HashSet<String>,

    /// Expected number of nodes to provision.
    expected: u32,

    /// Node provisioning has been requested already.
    requested: bool,

    /// The set of expected nodes is full and has been stable for this many cycles.
    #[serde(default, with = "time::serde::rfc3339::option")]
    stable: Option<OffsetDateTime>,

    /// Set of nodes in the group at first invocation time.
    starting_nodes: HashSet<String>,
}

impl Default for ProvisionNodesState {
    fn default() -> Self {
        Self {
            candidate_nodes: Default::default(),
            expected: 0,
            requested: false,
            stable: None,
            starting_nodes: Default::default(),
        }
    }
}
