use failure::ResultExt;

use replicante_agent_client::Client;
use replicante_data_models::Event;
use replicante_data_models::Node;

use replicante_data_store::store::Store;
use replicante_streams_events::EventsStream;

use super::ClusterIdentityChecker;
use super::Error;
use super::ErrorKind;
use super::Result;


/// Subset of fetcher logic that deals specifically with nodes.
pub(crate) struct NodeFetcher {
    events: EventsStream,
    store: Store,
}

impl NodeFetcher {
    pub(crate) fn new(events: EventsStream, store: Store) -> NodeFetcher {
        NodeFetcher { events, store }
    }

    pub(crate) fn process_node(
        &self,
        client: &Client,
        id_checker: &mut ClusterIdentityChecker,
    ) -> Result<()> {
        let info = client
            .datastore_info()
            .with_context(|_| ErrorKind::AgentRead("datastore info", client.id().to_string()))?;
        let node = Node::new(info);
        id_checker.check_id(&node.cluster_id, &node.node_id)?;
        id_checker.check_or_set_display_name(&node.cluster_display_name, &node.node_id)?;
        let cluster_id = node.cluster_id.clone();
        let node_id = node.node_id.clone();
        match self.store.node(cluster_id, node_id).get() {
            Err(error) => Err(error)
                .with_context(|_| ErrorKind::StoreRead("node")).map_err(Error::from),
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
        let code = event.code();
        self.events
            .emit(event)
            .with_context(|_| ErrorKind::EventEmit(code))?;
        self.store
            .persist()
            .node(node)
            .with_context(|_| ErrorKind::StoreWrite("node update")).map_err(Error::from)
    }

    fn process_node_new(&self, node: Node) -> Result<()> {
        let event = Event::builder().node().node_new(node.clone());
        let code = event.code();
        self.events
            .emit(event)
            .with_context(|_| ErrorKind::EventEmit(code))?;
        self.store
            .persist()
            .node(node)
            .with_context(|_| ErrorKind::StoreWrite("new node")).map_err(Error::from)
    }
}
