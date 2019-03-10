use failure::ResultExt;
use failure::SyncFailure;

use replicante_agent_client::Client;
use replicante_data_models::Event;
use replicante_data_models::Node;

use replicante_data_store::Store;
use replicante_streams_events::EventsStream;

use super::Error;
use super::ErrorKind;
use super::Result;
use super::meta::ClusterMetaBuilder;


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
        let info = client.datastore_info().map_err(SyncFailure::new)
            .with_context(|_| ErrorKind::AgentRead("datastore info", client.id().to_string()))?;
        let node = Node::new(info);
        meta.node_kind(node.kind.clone());

        let cluster = node.cluster.clone();
        let name = node.name.clone();
        match self.store.node(cluster, name) {
            Err(error) => Err(error).map_err(SyncFailure::new)
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
        self.events.emit(event).map_err(SyncFailure::new)
            .with_context(|_| ErrorKind::EventEmit(code))?;
        self.store.persist_node(node).map_err(SyncFailure::new)
            .with_context(|_| ErrorKind::StoreWrite("node update")).map_err(Error::from)
    }

    fn process_node_new(&self, node: Node) -> Result<()> {
        let event = Event::builder().node().node_new(node.clone());
        let code = event.code();
        self.events.emit(event).map_err(SyncFailure::new)
            .with_context(|_| ErrorKind::EventEmit(code))?;
        self.store.persist_node(node).map_err(SyncFailure::new)
            .with_context(|_| ErrorKind::StoreWrite("new node")).map_err(Error::from)
    }
}
