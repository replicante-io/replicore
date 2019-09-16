use failure::Fail;
use failure::ResultExt;
use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;

use replicante_externals_mongodb::operations::scan_collection;
use replicante_models_core::events::Event;

use super::constants::COLLECTION_EVENTS;
use super::document::EventDocument;
use crate::backend::DataInterface;
use crate::Cursor;
use crate::ErrorKind;
use crate::Result;

/// Data admin operations implementation using MongoDB.
pub struct Data {
    client: Client,
    db: String,
}

impl Data {
    pub fn new(client: Client, db: String) -> Data {
        Data { client, db }
    }
}

impl DataInterface for Data {
    fn events(&self) -> Result<Cursor<Event>> {
        let collection = self.client.db(&self.db).collection(COLLECTION_EVENTS);
        let cursor = scan_collection(collection)
            .with_context(|_| ErrorKind::MongoDBOperation)?
            .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()))
            .map(|result: Result<EventDocument>| result.map(Event::from));
        Ok(Cursor::new(cursor))
    }
}
