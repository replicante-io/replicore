use failure::Fail;
use failure::ResultExt;
use mongodb::bson;
use mongodb::bson::doc;
use mongodb::bson::Document;
use mongodb::options::FindOptions;
use mongodb::options::ReplaceOptions;
use mongodb::options::UpdateOptions;
use mongodb::results::UpdateResult;
use mongodb::sync::Collection;
use opentracingrust::SpanContext;
use opentracingrust::StartOptions;
use opentracingrust::Tracer;
use serde::de::DeserializeOwned;
use serde::Serialize;

use replicante_util_tracing::fail_span;

use crate::metrics::MONGODB_OPS_COUNT;
use crate::metrics::MONGODB_OPS_DURATION;
use crate::metrics::MONGODB_OP_ERRORS_COUNT;
use crate::Error;
use crate::ErrorKind;
use crate::Result;

/// Perform an [`aggregate`] operation.
///
/// [`aggregate`]: https://docs.mongodb.com/manual/reference/method/db.collection.aggregate/
pub fn aggregate<T>(
    collection: Collection<T>,
    pipeline: Vec<Document>,
    span: Option<SpanContext>,
    tracer: Option<&Tracer>,
) -> Result<impl Iterator<Item = Result<T>>>
where
    T: DeserializeOwned + Send + Sync,
{
    let mut span = match (tracer, span) {
        (Some(tracer), Some(context)) => {
            let opts = StartOptions::default().child_of(context);
            let mut span = tracer.span_with_options("store.mongodb.aggregate", opts);
            let namespace = collection.namespace();
            let namespace = format!("{}.{}", namespace.db, namespace.coll);
            span.tag("namespace", namespace);
            span.tag(
                "pipeline",
                serde_json::to_string(&pipeline)
                    .unwrap_or_else(|_| "<unable to encode pipeline>".into()),
            );
            Some(span.auto_finish())
        }
        _ => None,
    };
    MONGODB_OPS_COUNT.with_label_values(&["aggregate"]).inc();
    let timer = MONGODB_OPS_DURATION
        .with_label_values(&["aggregate"])
        .start_timer();
    let cursor = collection
        .aggregate(pipeline, None)
        .map_err(|error| {
            MONGODB_OP_ERRORS_COUNT
                .with_label_values(&["aggregate"])
                .inc();
            error
        })
        .with_context(|_| ErrorKind::AggregateOp)
        .map_err(|error| fail_span(error, span.as_deref_mut()))?;
    timer.observe_duration();
    drop(span);
    let cursor = cursor.map(|document| {
        let document = document.with_context(|_| ErrorKind::AggregateCursor)?;
        let id = document
            .get_object_id("_id")
            .map(bson::oid::ObjectId::to_hex)
            .unwrap_or_else(|_| "<NO ID>".into());
        bson::from_bson::<T>(bson::Bson::Document(document))
            .map_err(|error| error.context(ErrorKind::InvalidRecord(id)).into())
    });
    Ok(cursor)
}

/// Perform an [`deleteOne`] operation.
///
/// [`deleteOne`]: https://docs.mongodb.com/manual/reference/method/db.collection.deleteOne/
pub fn delete_one<T>(
    collection: Collection<T>,
    filter: Document,
    span: Option<SpanContext>,
    tracer: Option<&Tracer>,
) -> Result<()>
where
    T: Send + Sync,
{
    let mut span = match (tracer, span) {
        (Some(tracer), Some(context)) => {
            let opts = StartOptions::default().child_of(context);
            let mut span = tracer.span_with_options("store.mongodb.deleteOne", opts);
            let namespace = collection.namespace();
            let namespace = format!("{}.{}", namespace.db, namespace.coll);
            span.tag("namespace", namespace);
            span.tag(
                "filter",
                serde_json::to_string(&filter)
                    .unwrap_or_else(|_| "<unable to encode filter>".into()),
            );
            Some(span.auto_finish())
        }
        _ => None,
    };
    MONGODB_OPS_COUNT.with_label_values(&["deleteOne"]).inc();
    let _timer = MONGODB_OPS_DURATION
        .with_label_values(&["deleteOne"])
        .start_timer();
    collection
        .delete_one(filter, None)
        .map_err(|error| {
            MONGODB_OP_ERRORS_COUNT
                .with_label_values(&["deleteOne"])
                .inc();
            error
        })
        .with_context(|_| ErrorKind::DeleteOne)
        .map_err(|error| fail_span(error, span.as_deref_mut()))?;
    Ok(())
}

/// Perform a [`find`] operation.
///
/// [`find`]: https://docs.mongodb.com/manual/reference/method/db.collection.find/
pub fn find<T>(
    collection: Collection<T>,
    filter: Document,
    span: Option<SpanContext>,
    tracer: Option<&Tracer>,
) -> Result<impl Iterator<Item = Result<T>>>
where
    T: DeserializeOwned + Send + Sync + Unpin,
{
    let options = FindOptions::default();
    find_with_options(collection, filter, options, span, tracer)
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
pub fn find_one<T>(
    collection: Collection<T>,
    filter: Document,
    span: Option<SpanContext>,
    tracer: Option<&Tracer>,
) -> Result<Option<T>>
where
    T: DeserializeOwned + Send + Sync + Unpin,
{
    let mut span = match (tracer, span) {
        (Some(tracer), Some(context)) => {
            let options = StartOptions::default().child_of(context);
            let mut span = tracer.span_with_options("store.mongodb.findOne", options);
            let namespace = collection.namespace();
            let namespace = format!("{}.{}", namespace.db, namespace.coll);
            span.tag("namespace", namespace);
            span.tag(
                "filter",
                serde_json::to_string(&filter)
                    .unwrap_or_else(|_| "<unable to encode filter>".into()),
            );
            Some(span.auto_finish())
        }
        _ => None,
    };
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
        .with_context(|_| ErrorKind::FindOne)
        .map_err(|error| fail_span(error, span.as_deref_mut()))?;
    timer.observe_duration();
    drop(span);
    let document = match document {
        None => return Ok(None),
        Some(document) => document,
    };
    Ok(Some(document))
}

/// Perform a [`find`] operation with additional options.
///
/// [`find`]: https://docs.mongodb.com/manual/reference/method/db.collection.find/
pub fn find_with_options<T>(
    collection: Collection<T>,
    filter: Document,
    options: FindOptions,
    span: Option<SpanContext>,
    tracer: Option<&Tracer>,
) -> Result<impl Iterator<Item = Result<T>>>
where
    T: DeserializeOwned + Send + Sync + Unpin,
{
    let mut span = match (tracer, span) {
        (Some(tracer), Some(context)) => {
            let opts = StartOptions::default().child_of(context);
            let mut span = tracer.span_with_options("store.mongodb.find", opts);
            let namespace = collection.namespace();
            let namespace = format!("{}.{}", namespace.db, namespace.coll);
            span.tag("namespace", namespace);
            span.tag(
                "filter",
                serde_json::to_string(&filter)
                    .unwrap_or_else(|_| "<unable to encode filter>".into()),
            );
            span.tag(
                "options",
                serde_json::to_string(&options)
                    .unwrap_or_else(|_| "<unable to encode options>".into()),
            );
            Some(span.auto_finish())
        }
        _ => None,
    };
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
        .with_context(|_| ErrorKind::FindOp)
        .map_err(|error| fail_span(error, span.as_deref_mut()))?;
    timer.observe_duration();
    drop(span);
    let cursor = cursor.map(|document| {
        let document = document.with_context(|_| ErrorKind::FindCursor)?;
        Ok(document)
    });
    Ok(cursor)
}

/// Perform an [`insertMany`] operation.
///
/// [`insertMany`]: https://docs.mongodb.com/manual/reference/method/db.collection.insertMany/
pub fn insert_many<T>(
    collection: Collection<T>,
    records: Vec<T>,
    span: Option<SpanContext>,
    tracer: Option<&Tracer>,
) -> Result<()>
where
    T: Serialize + Send + Sync,
{
    let mut span = match (tracer, span) {
        (Some(tracer), Some(context)) => {
            let opts = StartOptions::default().child_of(context);
            let mut span = tracer.span_with_options("store.mongodb.insertMany", opts);
            let namespace = collection.namespace();
            let namespace = format!("{}.{}", namespace.db, namespace.coll);
            span.tag("namespace", namespace);
            Some(span.auto_finish())
        }
        _ => None,
    };
    MONGODB_OPS_COUNT.with_label_values(&["insertMany"]).inc();
    let _timer = MONGODB_OPS_DURATION
        .with_label_values(&["insertMany"])
        .start_timer();
    collection
        .insert_many(records, None)
        .map_err(|error| {
            MONGODB_OP_ERRORS_COUNT
                .with_label_values(&["insertMany"])
                .inc();
            error
        })
        .with_context(|_| ErrorKind::InsertMany)
        .map_err(|error| fail_span(error, span.as_deref_mut()))?;
    Ok(())
}

/// Perform an [`insertOne`] operation.
///
/// [`insertOne`]: https://docs.mongodb.com/manual/reference/method/db.collection.insertOne/
pub fn insert_one<T>(
    collection: Collection<T>,
    document: T,
    span: Option<SpanContext>,
    tracer: Option<&Tracer>,
) -> Result<()>
where
    T: Serialize + Send + Sync,
{
    let mut span = match (tracer, span) {
        (Some(tracer), Some(context)) => {
            let opts = StartOptions::default().child_of(context);
            let mut span = tracer.span_with_options("store.mongodb.insertOne", opts);
            let namespace = collection.namespace();
            let namespace = format!("{}.{}", namespace.db, namespace.coll);
            span.tag("namespace", namespace);
            span.tag(
                "document",
                serde_json::to_string(&document)
                    .unwrap_or_else(|_| "<unable to encode document>".into()),
            );
            Some(span.auto_finish())
        }
        _ => None,
    };
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
        .with_context(|_| ErrorKind::InsertOne)
        .map_err(|error| fail_span(error, span.as_deref_mut()))?;
    Ok(())
}

/// Perform an upserted [`replaceOne`] operation.
///
/// [`replaceOne`]: https://docs.mongodb.com/manual/reference/method/db.collection.replaceOne/
pub fn replace_one<T>(
    collection: Collection<T>,
    filter: Document,
    document: T,
    span: Option<SpanContext>,
    tracer: Option<&Tracer>,
) -> Result<()>
where
    T: Serialize + Send + Sync,
{
    let mut options = ReplaceOptions::default();
    options.upsert = Some(true);
    let mut span = match (tracer, span) {
        (Some(tracer), Some(context)) => {
            let options = StartOptions::default().child_of(context);
            let mut span = tracer.span_with_options("store.mongodb.replaceOne", options);
            let namespace = collection.namespace();
            let namespace = format!("{}.{}", namespace.db, namespace.coll);
            span.tag("namespace", namespace);
            span.tag(
                "filter",
                serde_json::to_string(&filter)
                    .unwrap_or_else(|_| "<unable to encode filter>".into()),
            );
            span.tag(
                "update",
                serde_json::to_string(&document)
                    .unwrap_or_else(|_| "<unable to encode update document>".into()),
            );
            Some(span.auto_finish())
        }
        _ => None,
    };
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
        .with_context(|_| ErrorKind::ReplaceOne)
        .map_err(|error| fail_span(error, span.as_deref_mut()))?;
    Ok(())
}

/// Scan all documents in a collection.
///
/// Intended for data validation purposes.
pub fn scan_collection<T>(collection: Collection<T>) -> Result<impl Iterator<Item = Result<T>>>
where
    T: DeserializeOwned + Send + Sync + Unpin,
{
    let filter = doc! {};
    let sort = doc! {"_id": 1};
    let mut options = FindOptions::default();
    options.sort = Some(sort);
    let cursor = find_with_options(collection, filter, options, None, None)
        .with_context(|_| ErrorKind::FindOp)?
        .map(|item| {
            let item = item.context(ErrorKind::FindCursor)?;
            Ok(item)
        });
    Ok(cursor)
}

/// Perform an [`updateMany`] operation.
///
/// [`updateMany`]: https://docs.mongodb.com/manual/reference/method/db.collection.updateMany/
pub fn update_many<T>(
    collection: Collection<T>,
    filter: Document,
    update: Document,
    span: Option<SpanContext>,
    tracer: Option<&Tracer>,
) -> Result<UpdateResult>
where
    T: Send + Sync,
{
    let mut span = match (tracer, span) {
        (Some(tracer), Some(context)) => {
            let options = StartOptions::default().child_of(context);
            let mut span = tracer.span_with_options("store.mongodb.updateMany", options);
            let namespace = collection.namespace();
            let namespace = format!("{}.{}", namespace.db, namespace.coll);
            span.tag("namespace", namespace);
            span.tag(
                "filter",
                serde_json::to_string(&filter)
                    .unwrap_or_else(|_| "<unable to encode filter>".into()),
            );
            span.tag(
                "update",
                serde_json::to_string(&update)
                    .unwrap_or_else(|_| "<unable to encode update document>".into()),
            );
            Some(span.auto_finish())
        }
        _ => None,
    };
    MONGODB_OPS_COUNT.with_label_values(&["updateMany"]).inc();
    let _timer = MONGODB_OPS_DURATION
        .with_label_values(&["updateMany"])
        .start_timer();
    collection
        .update_many(filter, update, None)
        .map_err(|error| {
            MONGODB_OP_ERRORS_COUNT
                .with_label_values(&["updateMany"])
                .inc();
            error
        })
        .with_context(|_| ErrorKind::UpdateMany)
        .map_err(|error| fail_span(error, span.as_deref_mut()))
        .map_err(Error::from)
}

/// Perform an [`updateOne`] operation.
///
/// [`updateOne`]: https://docs.mongodb.com/manual/reference/method/db.collection.updateOne/
pub fn update_one<T>(
    collection: Collection<T>,
    filter: Document,
    update: Document,
    span: Option<SpanContext>,
    tracer: Option<&Tracer>,
) -> Result<UpdateResult>
where
    T: Send + Sync,
{
    let options = UpdateOptions::default();
    update_one_with_options(collection, filter, update, options, span, tracer)
}

/// Perform an [`updateOne`] operation with additional options.
///
/// [`updateOne`]: https://docs.mongodb.com/manual/reference/method/db.collection.updateOne/
pub fn update_one_with_options<T>(
    collection: Collection<T>,
    filter: Document,
    update: Document,
    options: UpdateOptions,
    span: Option<SpanContext>,
    tracer: Option<&Tracer>,
) -> Result<UpdateResult> {
    let mut span = match (tracer, span) {
        (Some(tracer), Some(context)) => {
            let opts = StartOptions::default().child_of(context);
            let mut span = tracer.span_with_options("store.mongodb.updateOne", opts);
            let namespace = collection.namespace();
            let namespace = format!("{}.{}", namespace.db, namespace.coll);
            span.tag("namespace", namespace);
            span.tag(
                "filter",
                serde_json::to_string(&filter)
                    .unwrap_or_else(|_| "<unable to encode filter>".into()),
            );
            span.tag(
                "update",
                serde_json::to_string(&update)
                    .unwrap_or_else(|_| "<unable to encode update document>".into()),
            );
            Some(span.auto_finish())
        }
        _ => None,
    };
    MONGODB_OPS_COUNT.with_label_values(&["updateOne"]).inc();
    let _timer = MONGODB_OPS_DURATION
        .with_label_values(&["updateOne"])
        .start_timer();
    collection
        .update_one(filter, update, options)
        .map_err(|error| {
            MONGODB_OP_ERRORS_COUNT
                .with_label_values(&["updateOne"])
                .inc();
            error
        })
        .with_context(|_| ErrorKind::UpdateOne)
        .map_err(|error| fail_span(error, span.as_deref_mut()))
        .map_err(Error::from)
}
