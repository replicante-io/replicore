use failure::ResultExt;
use opentracingrust::Span;

use replicante_agent_client::Client;
use replicante_models_core::agent::Shard;
use replicante_models_core::events::Event;
use replicante_store_primary::store::Store;
use replicante_stream_events::EmitMessage;
use replicante_stream_events::Stream as EventsStream;

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
        cluster: &str,
        node: &str,
        span: &mut Span,
    ) -> Result<()> {
        let shards = client
            .shards(span.context().clone().into())
            .with_context(|_| ErrorKind::AgentDown("shards", client.id().to_string()))?;
        for shard in shards.shards {
            let shard = Shard::new(cluster.to_string(), node.to_string(), shard);
            self.process_shard(shard, span)?;
        }
        Ok(())
    }
}

impl ShardFetcher {
    fn process_shard(&self, shard: Shard, span: &mut Span) -> Result<()> {
        let cluster_id = shard.cluster_id.clone();
        let node_id = shard.node_id.clone();
        let shard_id = shard.shard_id.clone();
        let old = self
            .store
            .shard(cluster_id, node_id, shard_id)
            .get(span.context().clone());
        match old {
            Err(error) => Err(error)
                .with_context(|_| ErrorKind::StoreRead("shard"))
                .map_err(Error::from),
            Ok(None) => self.process_shard_new(shard, span),
            Ok(Some(old)) => self.process_shard_existing(shard, old, span),
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
            .with_context(|_| ErrorKind::StoreWrite("shard update"))
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
            .with_context(|_| ErrorKind::StoreWrite("new shard"))
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
