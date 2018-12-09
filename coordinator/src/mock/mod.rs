use std::sync::Arc;

use slog::Logger;

use super::Admin;
use super::Coordinator;
use super::NodeId;

mod admin;
mod backend;

use self::admin::MockAdmin;
use self::backend::MockBackend;


/// Helper to mock distributed coordination services.
pub struct MockCoordinator {
    pub node_id: NodeId,
}

impl MockCoordinator {
    pub fn new(_logger: Logger) -> MockCoordinator {
        MockCoordinator {
            node_id: NodeId::new(),
        }
    }

    pub fn admin(&self) -> Admin {
        Admin::with_backend(Arc::new(MockAdmin {
            // TODO
        }))
    }

    pub fn mock(&self) -> Coordinator {
        Coordinator::with_backend(Arc::new(MockBackend {
            node_id: self.node_id.clone(),
        }))
    }
}
