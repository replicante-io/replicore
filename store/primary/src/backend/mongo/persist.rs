use std::sync::Arc;

use bson::doc;
use bson::Bson;
use chrono::Utc;
use failure::ResultExt;
use mongodb::sync::Client;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use replicante_externals_mongodb::operations::replace_one;
use replicante_externals_mongodb::operations::update_one;
use replicante_models_core::actions::Action as ActionModel;
use replicante_models_core::agent::Agent as AgentModel;
use replicante_models_core::agent::AgentInfo as AgentInfoModel;
use replicante_models_core::agent::Node as NodeModel;
use replicante_models_core::agent::Shard as ShardModel;
use replicante_models_core::cluster::discovery::ClusterDiscovery as ClusterDiscoveryModel;
use replicante_models_core::cluster::discovery::DiscoverySettings as DiscoverySettingsModel;
use replicante_models_core::cluster::ClusterSettings as ClusterSettingsModel;

use super::super::PersistInterface;
use super::constants::COLLECTION_ACTIONS;
use super::constants::COLLECTION_AGENTS;
use super::constants::COLLECTION_AGENTS_INFO;
use super::constants::COLLECTION_CLUSTER_SETTINGS;
use super::constants::COLLECTION_DISCOVERIES;
use super::constants::COLLECTION_DISCOVERY_SETTINGS;
use super::constants::COLLECTION_NODES;
use super::constants::COLLECTION_SHARDS;
use super::document::ActionDocument;
use super::document::AgentInfoDocument;
use super::document::ClusterSettingsDocument;
use super::document::DiscoverySettingsDocument;
use super::document::NodeDocument;
use super::document::ShardDocument;
use crate::ErrorKind;
use crate::Result;

/// Persistence operations implementation using MongoDB.
pub struct Persist {
    client: Client,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl Persist {
    pub fn new<T>(client: Client, db: String, tracer: T) -> Persist
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        Persist { client, db, tracer }
    }
}

impl PersistInterface for Persist {
    fn action(&self, action: ActionModel, span: Option<SpanContext>) -> Result<()> {
        let action = ActionDocument::from(action);
        let filter = doc! {
            "cluster_id": &action.cluster_id,
            "node_id": &action.node_id,
            "action_id": &action.action_id,
        };
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_ACTIONS);
        let document = bson::to_bson(&action).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
        let document = match document {
            Bson::Document(document) => document,
            _ => panic!("Action failed to encode as BSON document"),
        };
        replace_one(collection, filter, document, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(())
    }

    fn agent(&self, agent: AgentModel, span: Option<SpanContext>) -> Result<()> {
        let filter = doc! {
            "cluster_id": &agent.cluster_id,
            "host": &agent.host,
        };
        let collection = self.client.database(&self.db).collection(COLLECTION_AGENTS);
        let document = bson::to_bson(&agent).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
        let document = match document {
            Bson::Document(document) => document,
            _ => panic!("Agent failed to encode as BSON document"),
        };
        replace_one(collection, filter, document, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(())
    }

    fn agent_info(&self, agent: AgentInfoModel, span: Option<SpanContext>) -> Result<()> {
        let filter = doc! {
            "cluster_id": &agent.cluster_id,
            "host": &agent.host,
        };
        let agent = AgentInfoDocument::from(agent);
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_AGENTS_INFO);
        let document = bson::to_bson(&agent).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
        let document = match document {
            Bson::Document(document) => document,
            _ => panic!("AgentInfo failed to encode as BSON document"),
        };
        replace_one(collection, filter, document, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(())
    }

    fn cluster_discovery(
        &self,
        discovery: ClusterDiscoveryModel,
        span: Option<SpanContext>,
    ) -> Result<()> {
        let filter = doc! {"cluster_id": &discovery.cluster_id};
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_DISCOVERIES);
        let document = bson::to_bson(&discovery).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
        let document = match document {
            Bson::Document(document) => document,
            _ => panic!("ClusterDiscovery failed to encode as BSON document"),
        };
        replace_one(collection, filter, document, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(())
    }

    fn cluster_settings(
        &self,
        settings: ClusterSettingsModel,
        span: Option<SpanContext>,
    ) -> Result<()> {
        let filter = doc! {
            "namespace": &settings.namespace,
            "cluster_id": &settings.cluster_id,
        };
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_CLUSTER_SETTINGS);
        let document = ClusterSettingsDocument::from(settings);
        let document = bson::to_bson(&document).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
        let document = match document {
            Bson::Document(document) => document,
            _ => panic!("ClusterSettings failed to encode as BSON document"),
        };
        replace_one(collection, filter, document, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(())
    }

    fn discovery_settings(
        &self,
        settings: DiscoverySettingsModel,
        span: Option<SpanContext>,
    ) -> Result<()> {
        let filter = doc! {
            "namespace": &settings.namespace,
            "name": &settings.name,
        };
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_DISCOVERY_SETTINGS);
        let settings = DiscoverySettingsDocument::from(settings);
        let document = bson::to_bson(&settings).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
        let document = match document {
            Bson::Document(document) => document,
            _ => panic!("DiscoverySettings failed to encode as BSON document"),
        };
        replace_one(collection, filter, document, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(())
    }

    fn next_cluster_orchestrate(
        &self,
        settings: ClusterSettingsModel,
        span: Option<SpanContext>,
    ) -> Result<()> {
        let filter = doc! {
            "namespace": &settings.namespace,
            "cluster_id": &settings.cluster_id,
        };
        let next_orchestrate = Utc::now() + chrono::Duration::seconds(settings.interval);
        let update = doc! {"$set": {"next_orchestrate": next_orchestrate}};
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_CLUSTER_SETTINGS);
        update_one(collection, filter, update, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(())
    }

    fn next_discovery_run(
        &self,
        settings: DiscoverySettingsModel,
        span: Option<SpanContext>,
    ) -> Result<()> {
        let filter = doc! {
            "namespace": &settings.namespace,
            "name": &settings.name,
        };
        let next_run = Utc::now() + chrono::Duration::seconds(settings.interval);
        let update = doc! {"$set": {"next_run": next_run}};
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_DISCOVERY_SETTINGS);
        update_one(collection, filter, update, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(())
    }

    fn node(&self, node: NodeModel, span: Option<SpanContext>) -> Result<()> {
        let filter = doc! {
            "cluster_id": &node.cluster_id,
            "node_id": &node.node_id,
        };
        let node = NodeDocument::from(node);
        let collection = self.client.database(&self.db).collection(COLLECTION_NODES);
        let document = bson::to_bson(&node).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
        let document = match document {
            Bson::Document(document) => document,
            _ => panic!("Node failed to encode as BSON document"),
        };
        replace_one(collection, filter, document, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(())
    }

    fn shard(&self, shard: ShardModel, span: Option<SpanContext>) -> Result<()> {
        let filter = doc! {
            "cluster_id": &shard.cluster_id,
            "node_id": &shard.node_id,
            "shard_id": &shard.shard_id,
        };
        let shard = ShardDocument::from(shard);
        let collection = self.client.database(&self.db).collection(COLLECTION_SHARDS);
        let document = bson::to_bson(&shard).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
        let document = match document {
            Bson::Document(document) => document,
            _ => panic!("Shard failed to encode as BSON document"),
        };
        replace_one(collection, filter, document, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(())
    }
}
