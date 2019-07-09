use std::fmt;

use failure::Backtrace;
use failure::Context;
use failure::Fail;

/// Error information returned by functions in case of errors.
#[derive(Debug)]
pub struct Error(Context<ErrorKind>);

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        self.0.get_context()
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error(inner)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error(Context::new(kind))
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
        self.0.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.0.backtrace()
    }

    fn name(&self) -> Option<&str> {
        self.kind().kind_name()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// Exhaustive list of possible errors emitted by this crate.
#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "unable to fetch from a MongoDB aggregate cursor")]
    AggregateCursor,

    #[fail(display = "MongoDB aggregate failed")]
    AggregateOp,

    #[fail(display = "unable to fetch from a MongoDB find cursor")]
    FindCursor,

    #[fail(display = "MongoDB findOne failed")]
    FindOne,

    #[fail(display = "MongoDB find failed")]
    FindOp,

    #[fail(display = "MongoDB insertOne failed")]
    InsertOne,

    #[fail(display = "found invalid record with id '{}'", _0)]
    InvalidRecord(String),

    #[fail(display = "failed to read MongoDB cursor for listCollections operation")]
    ListCollectionsCursor,

    #[fail(display = "MongoDB listCollections operation failed")]
    ListCollectionsOp,

    #[fail(display = "failed to read MongoDB cursor for listIndexes operation")]
    ListIndexesCursor,

    #[fail(display = "MongoDB listIndexes operation failed")]
    ListIndexesOp,

    #[fail(display = "MongoDB replaceOne failed")]
    ReplaceOne,

    #[fail(display = "MongoDB updateMany failed")]
    UpdateMany,

    #[fail(display = "unable to detect MongoDb version")]
    Version,
}

impl ErrorKind {
    fn kind_name(&self) -> Option<&str> {
        let name = match self {
            ErrorKind::AggregateCursor => "AggregateCursor",
            ErrorKind::AggregateOp => "AggregateOp",
            ErrorKind::FindCursor => "FindCursor",
            ErrorKind::FindOne => "FindOne",
            ErrorKind::FindOp => "FindOp",
            ErrorKind::InsertOne => "InsertOne",
            ErrorKind::InvalidRecord(_) => "InvalidRecord",
            ErrorKind::ListCollectionsCursor => "ListCollectionsCursor",
            ErrorKind::ListCollectionsOp => "ListCollectionsOp",
            ErrorKind::ListIndexesCursor => "ListIndexesCursor",
            ErrorKind::ListIndexesOp => "ListIndexesOp",
            ErrorKind::ReplaceOne => "ReplaceOne",
            ErrorKind::UpdateMany => "UpdateMany",
            ErrorKind::Version => "Version",
        };
        Some(name)
    }
}

/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;
