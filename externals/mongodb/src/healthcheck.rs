use mongodb::bson::doc;
use mongodb::sync::Client;

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
        let info = self
            .client
            .database("test")
            .run_command(doc! {"isMaster": 1}, None);
        let info = match info {
            Ok(info) => info,
            Err(error) => return HealthStatus::Failed(error.to_string()),
        };
        match info.get_bool("ismaster") {
            Ok(true) => HealthStatus::Healthy,
            Ok(false) => HealthStatus::Failed("primary node not found".to_string()),
            Err(error) => HealthStatus::Failed(error.to_string()),
        }
    }
}
