use error_chain::ChainedError;
use slog::Logger;

use replicante_agent_client::Client;
use replicante_data_models::Agent;
use replicante_data_models::AgentInfo;
use replicante_data_store::Store;

use super::Result;
use super::metrics::FETCHER_ERRORS_COUNT;


/// Subset of fetcher logic that deals specifically with agents.
pub struct AgentFetcher {
    logger: Logger,
    store: Store,
}

impl AgentFetcher {
    pub fn new(logger: Logger, store: Store) -> AgentFetcher {
        AgentFetcher {
            logger,
            store,
        }
    }

    pub fn persist_agent(&self, agent: Agent) {
        let cluster = agent.cluster.clone();
        let host = agent.host.clone();
        let old = match self.store.agent(cluster.clone(), host.clone()) {
            Ok(old) => old,
            Err(error) => {
                FETCHER_ERRORS_COUNT.with_label_values(&[&cluster]).inc();
                let error = error.display_chain().to_string();
                error!(
                    self.logger, "Failed to fetch agent";
                    "cluster" => cluster, "host" => host, "error" => error
                );
                return;
            }
        };

        // TODO: Emit agent events.

        if old != Some(agent.clone()) {
            match self.store.persist_agent(agent) {
                Ok(_) => (),
                Err(error) => {
                    FETCHER_ERRORS_COUNT.with_label_values(&[&cluster]).inc();
                    let error = error.display_chain().to_string();
                    error!(
                        self.logger, "Failed to persist agent";
                        "cluster" => cluster, "host" => host, "error" => error
                    );
                }
            };
        }
    }

    pub fn persist_agent_info(&self, agent: AgentInfo) {
        let cluster = agent.cluster.clone();
        let host = agent.host.clone();
        let old = match self.store.agent_info(cluster.clone(), host.clone()) {
            Ok(old) => old,
            Err(error) => {
                FETCHER_ERRORS_COUNT.with_label_values(&[&cluster]).inc();
                let error = error.display_chain().to_string();
                error!(
                    self.logger, "Failed to fetch agent info";
                    "cluster" => cluster, "host" => host, "error" => error
                );
                return;
            }
        };

        // TODO: Emit agent events.

        if old != Some(agent.clone()) {
            match self.store.persist_agent_info(agent) {
                Ok(_) => (),
                Err(error) => {
                    FETCHER_ERRORS_COUNT.with_label_values(&[&cluster]).inc();
                    let error = error.display_chain().to_string();
                    error!(
                        self.logger, "Failed to persist agent info";
                        "cluster" => cluster, "host" => host, "error" => error
                    );
                }
            };
        }
    }

    pub fn process_agent(&self, client: &Client, cluster: String, node: String) -> Result<()> {
        let info = client.info()?;
        let info = AgentInfo::new(cluster, node, info.agent);
        self.persist_agent_info(info);
        Ok(())
    }
}
