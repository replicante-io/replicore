use failure::ResultExt;
use opentracingrust::Span;

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
        span: &mut Span,
    ) -> Result<()> {
        let info = client
            .datastore_info(span.context().clone().into())
            .with_context(|_| ErrorKind::AgentRead("datastore info", client.id().to_string()))?;
        let node = Node::new(info);
        id_checker.check_id(&node.cluster_id, &node.node_id)?;
        if let Some(display_name) = node.cluster_display_name.as_ref() {
            id_checker.check_or_set_display_name(display_name, &node.node_id)?;
        }
        let cluster_id = node.cluster_id.clone();
        let node_id = node.node_id.clone();
        let record = self
            .store
            .node(cluster_id, node_id)
            .get(span.context().clone());
        match record {
            Err(error) => Err(error)
                .with_context(|_| ErrorKind::StoreRead("node"))
                .map_err(Error::from),
            Ok(None) => self.process_node_new(node, span),
            Ok(Some(old)) => self.process_node_existing(node, old, span),
        }
    }
}

impl NodeFetcher {
    fn process_node_existing(&self, node: Node, old: Node, span: &mut Span) -> Result<()> {
        if node != old {
            let event = Event::builder().node().changed(old, node.clone());
            let code = event.code();
            self.events
                .emit(event, span.context().clone())
                .with_context(|_| ErrorKind::EventEmit(code))?;
        }
        // ALWAYS persist the model, even unchanged, to clear the staleness state.
        self.store
            .persist()
            .node(node, span.context().clone())
            .with_context(|_| ErrorKind::StoreWrite("node update"))
            .map_err(Error::from)
    }

    fn process_node_new(&self, node: Node, span: &mut Span) -> Result<()> {
        let event = Event::builder().node().node_new(node.clone());
        let code = event.code();
        self.events
            .emit(event, span.context().clone())
            .with_context(|_| ErrorKind::EventEmit(code))?;
        self.store
            .persist()
            .node(node, span.context().clone())
            .with_context(|_| ErrorKind::StoreWrite("new node"))
            .map_err(Error::from)
    }
}
