use replicante_data_models::Agent;
use replicante_data_models::AgentInfo;
use replicante_data_models::ClusterMeta;
use replicante_data_models::ClusterDiscovery;
use replicante_data_models::Event;
use replicante_data_models::Node;
use replicante_data_models::Shard;

use super::super::super::Cursor;
use super::super::super::ErrorKind;
use super::super::super::Result;
use super::super::super::ValidationResult;
use super::super::super::validator::InnerValidator;


/// A mock implementation of the storage validator for tests.
pub struct MockValidator {
}

impl InnerValidator for MockValidator {
    fn agents(&self) -> Result<Cursor<Agent>> {
        Err(ErrorKind::MockNotYetImplemented("agents").into())
    }

    fn agents_count(&self) -> Result<u64> {
        Err(ErrorKind::MockNotYetImplemented("agents count").into())
    }

    fn agents_info(&self) -> Result<Cursor<AgentInfo>> {
        Err(ErrorKind::MockNotYetImplemented("agents info").into())
    }

    fn agents_info_count(&self) -> Result<u64> {
        Err(ErrorKind::MockNotYetImplemented("agents info count").into())
    }

    fn cluster_discoveries(&self) -> Result<Cursor<ClusterDiscovery>> {
        Err(ErrorKind::MockNotYetImplemented("clusters discoveries").into())
    }

    fn cluster_discoveries_count(&self) -> Result<u64> {
        Err(ErrorKind::MockNotYetImplemented("clusters discoveries count").into())
    }

    fn clusters_meta(&self) -> Result<Cursor<ClusterMeta>> {
        Err(ErrorKind::MockNotYetImplemented("clusters meta").into())
    }

    fn clusters_meta_count(&self) -> Result<u64> {
        Err(ErrorKind::MockNotYetImplemented("clusters meta count").into())
    }

    fn events(&self) -> Result<Cursor<Event>> {
        Err(ErrorKind::MockNotYetImplemented("events").into())
    }

    fn events_count(&self) -> Result<u64> {
        Err(ErrorKind::MockNotYetImplemented("events count").into())
    }

    fn indexes(&self) -> Result<Vec<ValidationResult>> {
        Err(ErrorKind::MockNotYetImplemented("indexes").into())
    }

    fn nodes(&self) -> Result<Cursor<Node>> {
        Err(ErrorKind::MockNotYetImplemented("nodes").into())
    }

    fn nodes_count(&self) -> Result<u64> {
        Err(ErrorKind::MockNotYetImplemented("nodes count").into())
    }

    fn removed(&self) -> Result<Vec<ValidationResult>> {
        Err(ErrorKind::MockNotYetImplemented("removed").into())
    }

    fn schema(&self) -> Result<Vec<ValidationResult>> {
        Err(ErrorKind::MockNotYetImplemented("schema").into())
    }

    fn shards(&self) -> Result<Cursor<Shard>> {
        Err(ErrorKind::MockNotYetImplemented("shards").into())
    }

    fn shards_count(&self) -> Result<u64> {
        Err(ErrorKind::MockNotYetImplemented("shards count").into())
    }

    fn version(&self) -> Result<String> {
        Err(ErrorKind::MockNotYetImplemented("version").into())
    }
}
