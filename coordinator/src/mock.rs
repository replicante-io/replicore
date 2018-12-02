use slog::Logger;

use super::Coordinator;


/// Helper to mock distributed coordination services.
pub struct MockCoordinator {}

impl MockCoordinator {
    pub fn new(_logger: Logger) -> MockCoordinator {
        MockCoordinator {}
    }

    pub fn mock(&self) -> Coordinator {
        Coordinator()
    }
}
