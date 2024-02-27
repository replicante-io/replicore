use serde::Deserialize;
use serde::Serialize;

pub use replicante_externals_mongodb::CommonConfig;

/// Persisted storage configuration options.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "backend", content = "options")]
pub enum Config {
    /// Persist data in mongodb (recommended, default).
    #[serde(rename = "mongodb")]
    MongoDB(MongoDBConfig),
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct MongoDBConfig {
    #[serde(flatten)]
    pub common: CommonConfig,

    /// Name of the MongoDB database to use for persistence.
    #[serde(default = "MongoDBConfig::default_db")]
    pub db: String,
}

impl MongoDBConfig {
    fn default_db() -> String {
        String::from("replicore")
    }
}
