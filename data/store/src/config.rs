use serde_derive::Deserialize;
use serde_derive::Serialize;

/// Persisted storage configuration options.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "backend", content = "options", deny_unknown_fields)]
pub enum Config {
    /// Persist data in mongodb (recommended, default).
    #[serde(rename = "mongodb")]
    MongoDB(MongoDBConfig),
}

impl Default for Config {
    fn default() -> Config {
        Config::MongoDB(MongoDBConfig::default())
    }
}

/// MongoDB storage configuration options.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct MongoDBConfig {
    #[serde(default = "MongoDBConfig::default_db")]
    pub db: String,

    #[serde(default = "MongoDBConfig::default_uri")]
    pub uri: String,
}

impl Default for MongoDBConfig {
    fn default() -> MongoDBConfig {
        MongoDBConfig {
            db: MongoDBConfig::default_db(),
            uri: MongoDBConfig::default_uri(),
        }
    }
}

impl MongoDBConfig {
    fn default_db() -> String {
        String::from("replicore")
    }
    fn default_uri() -> String {
        String::from("mongodb://localhost:27017/")
    }
}
