#[macro_use(bson, doc)]
extern crate bson;
extern crate config;
extern crate mongodb;

extern crate serde;
#[macro_use]
extern crate serde_derive;

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

pub mod settings;
mod error;

use self::settings::MongoDBSettings;


/// Agent dealing with MongoDB 3.x Replica Sets.
pub struct MongoDBAgent {
    settings: MongoDBSettings,
}

impl MongoDBAgent {
    pub fn new(settings: MongoDBSettings) -> MongoDBAgent {
        MongoDBAgent {
            settings: settings,
        }
    }
}

impl MongoDBAgent {
    /// Instantiates a client to interact with MongoDB.
    fn client(&self) -> AgentResult<Client> {
        let host = &self.settings.host;
        let port = self.settings.port as u16;
        Client::connect(host, port).map_err(self::error::to_agent)
    }
}

impl Agent for MongoDBAgent {
    fn datastore_version(&self) -> AgentResult<DatastoreVersion> {
        let mongo = self.client()?;
        let info = mongo.db("test").command(
            doc! {"buildInfo" => 1},
            CommandType::BuildInfo,
            None
        ).map_err(self::error::to_agent)?;
        let version = info.get("version").ok_or(AgentError::ModelViolation(
            String::from("Unable to determine MongoDB version")
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
