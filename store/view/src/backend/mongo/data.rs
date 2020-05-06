use failure::Fail;
use failure::ResultExt;
use mongodb::Client;

use replicante_externals_mongodb::operations::scan_collection;
use replicante_models_core::actions::Action;
use replicante_models_core::actions::ActionHistory;
use replicante_models_core::events::Event;

use super::constants::COLLECTION_ACTIONS;
use super::constants::COLLECTION_ACTIONS_HISTORY;
use super::constants::COLLECTION_EVENTS;
use super::document::ActionDocument;
use super::document::ActionHistoryDocument;
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
    fn actions(&self) -> Result<Cursor<Action>> {
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_ACTIONS);
        let cursor = scan_collection(collection)
            .with_context(|_| ErrorKind::MongoDBOperation)?
            .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()))
            .map(|result: Result<ActionDocument>| result.map(Action::from));
        Ok(Cursor::new(cursor))
    }

    fn actions_history(&self) -> Result<Cursor<ActionHistory>> {
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_ACTIONS_HISTORY);
        let cursor = scan_collection(collection)
            .with_context(|_| ErrorKind::MongoDBOperation)?
            .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()))
            .map(|result: Result<ActionHistoryDocument>| result.map(ActionHistory::from));
        Ok(Cursor::new(cursor))
    }

    fn events(&self) -> Result<Cursor<Event>> {
        let collection = self.client.database(&self.db).collection(COLLECTION_EVENTS);
        let cursor = scan_collection(collection)
            .with_context(|_| ErrorKind::MongoDBOperation)?
            .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()))
            .map(|result: Result<EventDocument>| result.map(Event::from));
        Ok(Cursor::new(cursor))
    }
}
