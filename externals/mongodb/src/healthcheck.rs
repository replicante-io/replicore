use mongodb::Client;
use mongodb::ThreadedClient;

use replicante_models_api::HealthStatus;
use replicante_service_healthcheck::HealthCheck;

pub struct MongoDBHealthCheck {
    client: Client,
}

impl MongoDBHealthCheck {
    pub fn new(client: Client) -> MongoDBHealthCheck {
        MongoDBHealthCheck { client }
    }
}

impl HealthCheck for MongoDBHealthCheck {
    fn check(&self) -> HealthStatus {
        match self.client.is_master() {
            Ok(_) => HealthStatus::Healthy,
            Err(error) => HealthStatus::Failed(error.to_string()),
        }
    }
}
