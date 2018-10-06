use replicante_agent_client::Client;
use replicante_data_models::Event;
use replicante_data_models::Node;

use replicante_data_store::Store;
use replicante_streams_events::EventsStream;

use super::Result;
use super::ResultExt;
use super::meta::ClusterMetaBuilder;


const FAIL_FIND_NODE: &str = "Failed to fetch node";
const FAIL_PERSIST_NODE: &str = "Failed to persist node";


/// Subset of fetcher logic that deals specifically with nodes.
pub struct NodeFetcher {
    events: EventsStream,
    store: Store,
}

impl NodeFetcher {
    pub fn new(events: EventsStream, store: Store) -> NodeFetcher {
        NodeFetcher {
            events,
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
        let event = Event::builder().node().changed(old, node.clone());
        self.events.emit(event).chain_err(|| FAIL_PERSIST_NODE)?;
        self.store.persist_node(node).chain_err(|| FAIL_PERSIST_NODE)
    }

    fn process_node_new(&self, node: Node) -> Result<()> {
        let event = Event::builder().node().node_new(node.clone());
        self.events.emit(event).chain_err(|| FAIL_PERSIST_NODE)?;
        self.store.persist_node(node).chain_err(|| FAIL_PERSIST_NODE)
    }
}
