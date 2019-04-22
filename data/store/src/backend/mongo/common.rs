use bson::ordered::OrderedDocument;
use failure::Fail;
use failure::ResultExt;
use mongodb::coll::options::FindOptions;
use mongodb::coll::options::UpdateOptions;
use mongodb::coll::Collection;
use serde::Deserialize;

use super::super::super::Cursor;
use super::super::super::ErrorKind;
use super::super::super::Result;
use super::metrics::MONGODB_OPS_COUNT;
use super::metrics::MONGODB_OPS_DURATION;
use super::metrics::MONGODB_OP_ERRORS_COUNT;

/// Perform a [`find`] operation.
///
/// [`find`]: https://docs.mongodb.com/manual/reference/method/db.collection.find/
pub fn find<'de, T>(collection: Collection, filter: OrderedDocument) -> Result<Cursor<T>>
where
    T: Deserialize<'de>,
{
    let options = FindOptions::default();
    find_with_options(collection, filter, options)
}

/// Perform a [`find`] operation with additional options.
///
/// [`find`]: https://docs.mongodb.com/manual/reference/method/db.collection.find/
pub fn find_with_options<'de, T>(
    collection: Collection,
    filter: OrderedDocument,
    options: FindOptions,
) -> Result<Cursor<T>>
where
    T: Deserialize<'de>,
{
    MONGODB_OPS_COUNT.with_label_values(&["find"]).inc();
    let timer = MONGODB_OPS_DURATION
        .with_label_values(&["find"])
        .start_timer();
    let cursor = collection
        .find(Some(filter), Some(options))
        .map_err(|error| {
            MONGODB_OP_ERRORS_COUNT.with_label_values(&["find"]).inc();
            error
        })
        .with_context(|_| ErrorKind::MongoDBOperation("find"))?;
    timer.observe_duration();
    let iter = cursor.map(|document| {
        let document = document.with_context(|_| ErrorKind::MongoDBCursor("find"))?;
        let id = document
            .get_object_id("_id")
            .map(bson::oid::ObjectId::to_hex)
            .unwrap_or_else(|_| "<NO ID>".into());
        bson::from_bson::<T>(bson::Bson::Document(document))
            .map_err(|error| error.context(ErrorKind::InvalidRecord(id)).into())
    });
    Ok(Cursor(Box::new(iter)))
}

/// Perform a [`findOne`] operation.
///
/// # Return
/// If a document is found, attempt to BSON-decoded it to the requested model.
///
///  * `Err(error)` if the operation failed.
///  * `Ok(None)` if the operation suceeded but no document is returned.
///  * `Ok(Some(document))` if the operation succeeded and `document` was found.
///
/// [`findOne`]: https://docs.mongodb.com/manual/reference/method/db.collection.findOne/
pub fn find_one<'de, T>(collection: Collection, filter: OrderedDocument) -> Result<Option<T>>
where
    T: Deserialize<'de>,
{
    MONGODB_OPS_COUNT.with_label_values(&["findOne"]).inc();
    let timer = MONGODB_OPS_DURATION
        .with_label_values(&["findOne"])
        .start_timer();
    let document = collection
        .find_one(Some(filter), None)
        .map_err(|error| {
            MONGODB_OP_ERRORS_COUNT
                .with_label_values(&["findOne"])
                .inc();
            error
        })
        .with_context(|_| ErrorKind::MongoDBOperation("findOne"))?;
    timer.observe_duration();
    if document.is_none() {
        return Ok(None);
    }
    let document = document.unwrap();
    let id = document
        .get_object_id("_id")
        .map(bson::oid::ObjectId::to_hex)
        .unwrap_or_else(|_| "<NO ID>".into());
    let document = bson::from_bson::<T>(bson::Bson::Document(document))
        .map_err(|error| error.context(ErrorKind::InvalidRecord(id)))?;
    Ok(Some(document))
}

/// Perform an [`insertOne`] operation.
///
/// [`insertOne`]: https://docs.mongodb.com/manual/reference/method/db.collection.insertOne/
pub fn insert_one(collection: Collection, document: OrderedDocument) -> Result<()> {
    MONGODB_OPS_COUNT.with_label_values(&["insertOne"]).inc();
    let _timer = MONGODB_OPS_DURATION
        .with_label_values(&["insertOne"])
        .start_timer();
    collection
        .insert_one(document, None)
        .map_err(|error| {
            MONGODB_OP_ERRORS_COUNT
                .with_label_values(&["insertOne"])
                .inc();
            error
        })
        .with_context(|_| ErrorKind::MongoDBOperation("insertOne"))?;
    Ok(())
}

/// Perform an upserted [`replaceOne`] operation.
///
/// [`replaceOne`]: https://docs.mongodb.com/manual/reference/method/db.collection.replaceOne/
pub fn replace_one(
    collection: Collection,
    filter: OrderedDocument,
    document: OrderedDocument,
) -> Result<()> {
    let mut options = UpdateOptions::new();
    options.upsert = Some(true);
    MONGODB_OPS_COUNT.with_label_values(&["replaceOne"]).inc();
    let _timer = MONGODB_OPS_DURATION
        .with_label_values(&["replaceOne"])
        .start_timer();
    collection
        .replace_one(filter, document, Some(options))
        .map_err(|error| {
            MONGODB_OP_ERRORS_COUNT
                .with_label_values(&["replaceOne"])
                .inc();
            error
        })
        .with_context(|_| ErrorKind::MongoDBOperation("replaceOne"))?;
    Ok(())
}
