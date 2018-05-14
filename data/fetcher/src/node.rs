use replicante_agent_client::Client;
use replicante_data_models::Node;
use replicante_data_store::Store;

use super::Result;
use super::ResultExt;
use super::meta::ClusterMetaBuilder;


const FAIL_FIND_NODE: &'static str = "Failed to fetch node";
const FAIL_PERSIST_NODE: &'static str = "Failed to persist node";


/// Subset of fetcher logic that deals specifically with nodes.
pub struct NodeFetcher {
    store: Store,
}

impl NodeFetcher {
    pub fn new(store: Store) -> NodeFetcher {
        NodeFetcher {
            store,
        }
    }

    pub fn process_node(&self, client: &Client, meta: &mut ClusterMetaBuilder) -> Result<()> {
        let info = client.datastore_info()?;
        let node = Node::new(info);
        meta.node_kind(node.kind.clone());

        let cluster = node.cluster.clone();
        let name = node.name.clone();
        match self.store.node(cluster, name) {
            Err(error) => Err(error).chain_err(|| FAIL_FIND_NODE),
            Ok(None) => self.process_node_new(node),
            Ok(Some(old)) => self.process_node_existing(node, old),
        }
    }
}

impl NodeFetcher {
    fn process_node_existing(&self, node: Node, old: Node) -> Result<()> {
        if node == old {
            return Ok(());
        }
        // TODO(stefano): emit node changed events.
        self.store.persist_node(node).chain_err(|| FAIL_PERSIST_NODE)
    }

    fn process_node_new(&self, node: Node) -> Result<()> {
        // TODO(stefano): emit node new events.
        self.store.persist_node(node).chain_err(|| FAIL_PERSIST_NODE)
    }
}
