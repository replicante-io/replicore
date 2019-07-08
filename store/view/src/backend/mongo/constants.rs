use bson::bson;
use bson::doc;
use bson::ordered::OrderedDocument;
use lazy_static::lazy_static;

pub const COLLECTION_EVENTS: &str = "events";

lazy_static! {
    pub static ref EVENTS_FILTER_NOT_SNAPSHOT: OrderedDocument = doc! {"$nin" => [
        "SNAPSHOT_AGENT",
        "SNAPSHOT_AGENT_INFO",
        "SNAPSHOT_DISCOVERY",
        "SNAPSHOT_NODE",
        "SNAPSHOT_SHARD",
    ]};
}
