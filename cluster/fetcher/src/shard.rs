use failure::ResultExt;
use opentracingrust::Span;

use replicante_agent_client::Client;
use replicante_models_core::agent::Shard;
use replicante_models_core::events::Event;
use replicante_store_primary::store::Store;
use replicante_stream_events::EmitMessage;
use replicante_stream_events::Stream as EventsStream;

use replicore_cluster_view::ClusterView;
use replicore_cluster_view::ClusterViewBuilder;

use super::Error;
use super::ErrorKind;
use super::Result;

/// Subset of fetcher logic that deals specifically with shards.
pub(crate) struct ShardFetcher {
    events: EventsStream,
    store: Store,
}

impl ShardFetcher {
    pub(crate) fn new(events: EventsStream, store: Store) -> ShardFetcher {
        ShardFetcher { events, store }
    }

    pub(crate) fn process_shards(
        &self,
        client: &dyn Client,
        cluster_view: &ClusterView,
        new_cluster_view: &mut ClusterViewBuilder,
        node: &str,
        span: &mut Span,
    ) -> Result<()> {
        let shards = client
            .shards(span.context().clone().into())
            .with_context(|_| ErrorKind::AgentDown("shards", client.id().to_string()))?;
        for shard in shards.shards {
            let shard = Shard::new(cluster_view.cluster_id.clone(), node.to_string(), shard);
            self.process_shard(cluster_view, new_cluster_view, shard, span)?;
        }
        Ok(())
    }
}

impl ShardFetcher {
    fn process_shard(
        &self,
        cluster_view: &ClusterView,
        new_cluster_view: &mut ClusterViewBuilder,
        shard: Shard,
        span: &mut Span,
    ) -> Result<()> {
        new_cluster_view
            .shard(shard.clone())
            .map_err(crate::error::AnyWrap::from)
            .context(ErrorKind::ClusterViewUpdate)?;
        let old = cluster_view
            .shard_on_node(&shard.node_id, &shard.shard_id)
            .cloned();
        match old {
            None => self.process_shard_new(shard, span),
            Some(old) => self.process_shard_existing(shard, old, span),
        }
    }

    fn process_shard_existing(&self, shard: Shard, old: Shard, span: &mut Span) -> Result<()> {
        // If anything other then offset or lag changed emit and event.
        if self.shard_changed(&shard, &old) {
            let event = Event::builder()
                .shard()
                .allocation_changed(old, shard.clone());
            let code = event.code();
            let stream_key = event.stream_key();
            let event = EmitMessage::with(stream_key, event)
                .with_context(|_| ErrorKind::EventEmit(code))?
                .trace(span.context().clone());
            self.events
                .emit(event)
                .with_context(|_| ErrorKind::EventEmit(code))?;
        }

        // Persist the model so the latest offset and lag information are available.
        // ALWAYS persist the model, even unchanged, to clear the staleness state.
        self.store
            .persist()
            .shard(shard, span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStoreWrite("shard update"))
            .map_err(Error::from)
    }

    fn process_shard_new(&self, shard: Shard, span: &mut Span) -> Result<()> {
        let event = Event::builder().shard().new_allocation(shard.clone());
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
            .shard(shard, span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStoreWrite("new shard"))
            .map_err(Error::from)
    }

    /// Checks if a shard has changed since the last fetch.
    ///
    /// Because shard data includes commit offsets and lag we need to do a more
    /// in-depth comparison to ignore "expected" changes.
    fn shard_changed(&self, shard: &Shard, old: &Shard) -> bool {
        // Easy case: they are the same.
        if shard == old {
            return false;
        }
        // Check if the "stable" attributes have changed.
        shard.cluster_id != old.cluster_id
            || shard.node_id != old.node_id
            || shard.role != old.role
            || shard.shard_id != old.shard_id
    }
}
