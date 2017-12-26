#[macro_use(bson, doc)]
extern crate bson;
extern crate mongodb;

extern crate unamed_agent;

use bson::Bson;
use mongodb::Client;
use mongodb::CommandType;
use mongodb::ThreadedClient;
use mongodb::db::ThreadedDatabase;

use unamed_agent::Agent;
use unamed_agent::AgentError;
use unamed_agent::AgentResult;
use unamed_agent::models::DatastoreVersion;

mod error;


pub struct MongoDBAgent {
    mongo: Client
}

impl MongoDBAgent {
    pub fn new() -> MongoDBAgent {
        // TODO: load configuration.
        // TODO: connect to client.
        MongoDBAgent {
            mongo: Client::connect("172.17.0.2", 27017).expect("Failed to connect to MongoDB")
        }
    }
}

impl Agent for MongoDBAgent {
    fn datastore_version(&self) -> AgentResult<DatastoreVersion> {
        let info = self.mongo.db("test").command(
            doc! {"buildInfo" => 1},
            CommandType::BuildInfo,
            None
        );
        match info {
            Err(error) => Err(self::error::to_agent(error)),
            Ok(info) => {
                let version = info.get("version").ok_or(AgentError::ModelViolation(
                    String::from("Datastore does not expose its version")
                ))?;
                if let &Bson::String(ref version) = version {
                    Ok(DatastoreVersion::new("MongoDB", version))
                } else {
                    Err(AgentError::ModelViolation(String::from(
                        "Unexpeted version type (should be String)"
                    )))
                }
            }
        }
    }
}
