use replicante_agent_client::Client;
use replicante_data_models::Agent;
use replicante_data_models::AgentInfo;
use replicante_data_store::Store;

use super::Result;
use super::ResultExt;


const FAIL_FIND_AGENT: &'static str = "Failed to fetch agent";
const FAIL_FIND_AGENT_INFO: &'static str = "Failed to fetch agent info";
const FAIL_PERSIST_AGENT: &'static str = "Failed to persist agent";
const FAIL_PERSIST_AGENT_INFO: &'static str = "Failed to persist agent info";


/// Subset of fetcher logic that deals specifically with agents.
pub struct AgentFetcher {
    store: Store,
}

impl AgentFetcher {
    pub fn new(store: Store) -> AgentFetcher {
        AgentFetcher {
            store,
        }
    }

    pub fn process_agent(&self, agent: Agent) -> Result<()> {
        match self.store.agent(agent.cluster.clone(), agent.host.clone()) {
            Err(error) => Err(error).chain_err(|| FAIL_FIND_AGENT),
            Ok(None) => self.process_agent_new(agent),
            Ok(Some(old)) => self.process_agent_existing(agent, old),
        }
    }

    pub fn process_agent_info(&self, client: &Client, cluster: String, node: String) -> Result<()> {
        let info = client.info()?;
        let info = AgentInfo::new(cluster, node, info.agent);
        match self.store.agent_info(info.cluster.clone(), info.host.clone()) {
            Err(error) => Err(error).chain_err(|| FAIL_FIND_AGENT_INFO),
            Ok(None) => self.process_agent_info_new(info),
            Ok(Some(old)) => self.process_agent_info_existing(info, old),
        }
    }
}

impl AgentFetcher {
    fn process_agent_existing(&self, agent: Agent, old: Agent) -> Result<()> {
        // TODO: emit events.
        if agent == old {
            return Ok(());
        }
        self.store.persist_agent(agent).chain_err(|| FAIL_PERSIST_AGENT)
    }

    fn process_agent_new(&self, agent: Agent) -> Result<()> {
        // TODO: emit events.
        self.store.persist_agent(agent).chain_err(|| FAIL_PERSIST_AGENT)
    }

    fn process_agent_info_existing(&self, agent: AgentInfo, old: AgentInfo) -> Result<()> {
        // TODO: emit events.
        if agent == old {
            return Ok(());
        }
        self.store.persist_agent_info(agent).chain_err(|| FAIL_PERSIST_AGENT_INFO)
    }

    fn process_agent_info_new(&self, agent: AgentInfo) -> Result<()> {
        // TODO: emit events.
        self.store.persist_agent_info(agent).chain_err(|| FAIL_PERSIST_AGENT_INFO)
    }
}
