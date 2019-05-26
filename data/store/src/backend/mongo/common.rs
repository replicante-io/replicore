use bson::ordered::OrderedDocument;
use failure::Fail;
use failure::ResultExt;
use mongodb::coll::options::FindOptions;
use mongodb::coll::options::UpdateOptions;
use mongodb::coll::results::UpdateResult;
use mongodb::coll::Collection;
use mongodb::Document;
use opentracingrust::SpanContext;
use opentracingrust::StartOptions;
use opentracingrust::Tracer;
use serde::Deserialize;

use replicante_util_tracing::fail_span;

use super::super::super::Cursor;
use super::super::super::Error;
use super::super::super::ErrorKind;
use super::super::super::Result;
use super::metrics::MONGODB_OPS_COUNT;
use super::metrics::MONGODB_OPS_DURATION;
use super::metrics::MONGODB_OP_ERRORS_COUNT;

/// Perform an [`aggregate`] operation.
///
/// [`aggregate`]: https://docs.mongodb.com/manual/reference/method/db.collection.aggregate/
pub fn aggregate<'de, T>(
    collection: Collection,
    pipeline: Vec<OrderedDocument>,
    span: Option<SpanContext>,
    tracer: Option<&Tracer>,
) -> Result<Cursor<T>>
where
    T: Deserialize<'de>,
{
    let mut span = match (tracer, span) {
        (Some(tracer), Some(context)) => {
            let opts = StartOptions::default().child_of(context);
            let mut span = tracer.span_with_options("aggregate", opts);
            span.tag("namespace", collection.namespace.clone());
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
        .with_context(|_| ErrorKind::MongoDBOperation("aggregate"))
        .map_err(|error| match span {
            Some(ref mut span) => fail_span(error, span),
            None => error,
        })?;
    timer.observe_duration();
    drop(span);
    let iter = cursor.map(|document| {
        let document = document.with_context(|_| ErrorKind::MongoDBCursor("aggregate"))?;
        let id = document
            .get_object_id("_id")
            .map(bson::oid::ObjectId::to_hex)
            .unwrap_or_else(|_| "<NO ID>".into());
        bson::from_bson::<T>(bson::Bson::Document(document))
            .map_err(|error| error.context(ErrorKind::InvalidRecord(id)).into())
    });
    Ok(Cursor(Box::new(iter)))
}

/// Perform a [`find`] operation.
///
/// [`find`]: https://docs.mongodb.com/manual/reference/method/db.collection.find/
pub fn find<'de, T>(
    collection: Collection,
    filter: OrderedDocument,
    span: Option<SpanContext>,
    tracer: Option<&Tracer>,
) -> Result<Cursor<T>>
where
    T: Deserialize<'de>,
{
    let options = FindOptions::default();
    find_with_options(collection, filter, options, span, tracer)
}

/// Perform a [`find`] operation with additional options.
///
/// [`find`]: https://docs.mongodb.com/manual/reference/method/db.collection.find/
pub fn find_with_options<'de, T>(
    collection: Collection,
    filter: OrderedDocument,
    options: FindOptions,
    span: Option<SpanContext>,
    tracer: Option<&Tracer>,
) -> Result<Cursor<T>>
where
    T: Deserialize<'de>,
{
    let mut span = match (tracer, span) {
        (Some(tracer), Some(context)) => {
            let opts = StartOptions::default().child_of(context);
            let mut span = tracer.span_with_options("find", opts);
            span.tag(
                "filter",
                serde_json::to_string(&filter)
                    .unwrap_or_else(|_| "<unable to encode filter>".into()),
            );
            span.tag("namespace", collection.namespace.clone());
            span.tag(
                "options",
                serde_json::to_string(&Document::from(options.clone()))
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
        .with_context(|_| ErrorKind::MongoDBOperation("find"))
        .map_err(|error| match span {
            Some(ref mut span) => fail_span(error, span),
            None => error,
        })?;
    timer.observe_duration();
    drop(span);
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
pub fn find_one<'de, T>(
    collection: Collection,
    filter: OrderedDocument,
    span: Option<SpanContext>,
    tracer: Option<&Tracer>,
) -> Result<Option<T>>
where
    T: Deserialize<'de>,
{
    let mut span = match (tracer, span) {
        (Some(tracer), Some(context)) => {
            let options = StartOptions::default().child_of(context);
            let mut span = tracer.span_with_options("findOne", options);
            span.tag(
                "filter",
                serde_json::to_string(&filter)
                    .unwrap_or_else(|_| "<unable to encode filter>".into()),
            );
            span.tag("namespace", collection.namespace.clone());
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
        .with_context(|_| ErrorKind::MongoDBOperation("findOne"))
        .map_err(|error| match span {
            Some(ref mut span) => fail_span(error, span),
            None => error,
        })?;
    timer.observe_duration();
    drop(span);
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
pub fn insert_one(
    collection: Collection,
    document: OrderedDocument,
    span: Option<SpanContext>,
    tracer: Option<&Tracer>,
) -> Result<()> {
    let mut span = match (tracer, span) {
        (Some(tracer), Some(context)) => {
            let opts = StartOptions::default().child_of(context);
            let mut span = tracer.span_with_options("insertOne", opts);
            span.tag(
                "document",
                serde_json::to_string(&document)
                    .unwrap_or_else(|_| "<unable to encode document>".into()),
            );
            span.tag("namespace", collection.namespace.clone());
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
        .with_context(|_| ErrorKind::MongoDBOperation("insertOne"))
        .map_err(|error| match span {
            Some(ref mut span) => fail_span(error, span),
            None => error,
        })?;
    Ok(())
}

/// Perform an upserted [`replaceOne`] operation.
///
/// [`replaceOne`]: https://docs.mongodb.com/manual/reference/method/db.collection.replaceOne/
pub fn replace_one(
    collection: Collection,
    filter: OrderedDocument,
    document: OrderedDocument,
    span: Option<SpanContext>,
    tracer: Option<&Tracer>,
) -> Result<()> {
    let mut options = UpdateOptions::new();
    options.upsert = Some(true);
    let mut span = match (tracer, span) {
        (Some(tracer), Some(context)) => {
            let options = StartOptions::default().child_of(context);
            let mut span = tracer.span_with_options("replaceOne", options);
            span.tag(
                "filter",
                serde_json::to_string(&filter)
                    .unwrap_or_else(|_| "<unable to encode filter>".into()),
            );
            span.tag("namespace", collection.namespace.clone());
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
        .with_context(|_| ErrorKind::MongoDBOperation("replaceOne"))
        .map_err(|error| match span {
            Some(ref mut span) => fail_span(error, span),
            None => error,
        })?;
    Ok(())
}

/// Perform an [`updateMany`] operation.
///
/// [`updateMany`]: https://docs.mongodb.com/manual/reference/method/db.collection.updateMany/
pub fn update_many(
    collection: Collection,
    filter: OrderedDocument,
    update: OrderedDocument,
    span: Option<SpanContext>,
    tracer: Option<&Tracer>,
) -> Result<UpdateResult> {
    let mut span = match (tracer, span) {
        (Some(tracer), Some(context)) => {
            let options = StartOptions::default().child_of(context);
            let mut span = tracer.span_with_options("updateMany", options);
            span.tag(
                "filter",
                serde_json::to_string(&filter)
                    .unwrap_or_else(|_| "<unable to encode filter>".into()),
            );
            span.tag("namespace", collection.namespace.clone());
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
        .with_context(|_| ErrorKind::MongoDBOperation("updateMany"))
        .map_err(|error| match span {
            Some(ref mut span) => fail_span(error, span),
            None => error,
        })
        .map_err(Error::from)
}
