use failure::ResultExt;
use opentracingrust::Span;

use replicante_agent_client::Client;
use replicante_models_core::agent::Node;
use replicante_models_core::events::Event;
use replicante_store_primary::store::Store;
use replicante_stream_events::EmitMessage;
use replicante_stream_events::Stream as EventsStream;

use replicore_cluster_view::ClusterView;
use replicore_cluster_view::ClusterViewBuilder;

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
        client: &dyn Client,
        cluster_view: &ClusterView,
        new_cluster_view: &mut ClusterViewBuilder,
        id_checker: &mut ClusterIdentityChecker,
        span: &mut Span,
    ) -> Result<()> {
        let info = client
            .datastore_info(span.context().clone().into())
            .with_context(|_| {
                ErrorKind::DatastoreDown("datastore info", client.id().to_string())
            })?;
        let node = Node::new(info);
        id_checker.check_id(&node.cluster_id, &node.node_id)?;
        if let Some(display_name) = node.cluster_display_name.as_ref() {
            id_checker.check_or_set_display_name(display_name, &node.node_id)?;
        }
        new_cluster_view
            .node(node.clone())
            .map_err(crate::error::AnyWrap::from)
            .context(ErrorKind::ClusterViewUpdate)?;
        let old = cluster_view.node(&node.node_id).cloned();
        match old {
            None => self.process_node_new(node, span),
            Some(old) => self.process_node_existing(node, old, span),
        }
    }
}

impl NodeFetcher {
    fn process_node_existing(&self, node: Node, old: Node, span: &mut Span) -> Result<()> {
        if node != old {
            let event = Event::builder().node().changed(old, node.clone());
            let code = event.code();
            let stream_key = event.stream_key();
            let event = EmitMessage::with(stream_key, event)
                .with_context(|_| ErrorKind::EventEmit(code))?
                .trace(span.context().clone());
            self.events
                .emit(event)
                .with_context(|_| ErrorKind::EventEmit(code))?;
        }
        // ALWAYS persist the model, even unchanged, to clear the staleness state.
        self.store
            .persist()
            .node(node, span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStoreWrite("node update"))
            .map_err(Error::from)
    }

    fn process_node_new(&self, node: Node, span: &mut Span) -> Result<()> {
        let event = Event::builder().node().new_node(node.clone());
        let code = event.code();
        let stream_key = event.stream_key();
        let event = EmitMessage::with(stream_key, event)
            .with_context(|_| ErrorKind::EventEmit(code))?
            .trace(span.context().clone());
        self.events
            .emit(event)
            .with_context(|_| ErrorKind::EventEmit(code))?;
        self.store
            .persist()
            .node(node, span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStoreWrite("new node"))
            .map_err(Error::from)
    }
}
