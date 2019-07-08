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
    #[fail(display = "unexpected duplicate {} record with id '{}' found", _0, _1)]
    DuplicateRecord(&'static str, String),

    #[fail(display = "found invalid record with id '{}'", _0)]
    InvalidRecord(String),

    #[fail(display = "MongoDB BSON decode failed")]
    MongoDBBsonDecode,

    #[fail(display = "MongoDB BSON encode failed")]
    MongoDBBsonEncode,

    #[fail(display = "failed to connect to MongoDB URI '{}'", _0)]
    MongoDBConnect(String),

    #[fail(display = "failed to read MongoDB cursor")]
    MongoDBCursor,

    #[fail(display = "failed to read MongoDB cursor for listCollections operation")]
    MongoDBListCollectionsCursor,

    #[fail(display = "MongoDB listCollections operation failed")]
    MongoDBListCollectionsOp,

    #[fail(display = "failed to read MongoDB cursor for listIndexes operation")]
    MongoDBListIndexesCursor,

    #[fail(display = "MongoDB listIndexes operation failed")]
    MongoDBListIndexesOp,

    #[fail(display = "MongoDB operation failed")]
    MongoDBOperation,

    #[fail(display = "MongoDB version operation failed")]
    MongoDBVersion,

    #[fail(display = "{} record with id '{}' not found", _0, _1)]
    RecordNotFound(&'static str, String),
}

impl ErrorKind {
    fn kind_name(&self) -> Option<&str> {
        let name = match self {
            ErrorKind::DuplicateRecord(_, _) => "DuplicateRecord",
            ErrorKind::InvalidRecord(_) => "InvalidRecord",
            ErrorKind::MongoDBBsonDecode => "MongoDBBsonDecode",
            ErrorKind::MongoDBBsonEncode => "MongoDBBsonEncode",
            ErrorKind::MongoDBConnect(_) => "MongoDBConnect",
            ErrorKind::MongoDBCursor => "MongoDBCursor",
            ErrorKind::MongoDBListCollectionsCursor => "MongoDBListCollectionsCursor",
            ErrorKind::MongoDBListCollectionsOp => "MongoDBListCollectionsOp",
            ErrorKind::MongoDBListIndexesCursor => "MongoDBListIndexesCursor",
            ErrorKind::MongoDBListIndexesOp => "MongoDBListIndexesOp",
            ErrorKind::MongoDBOperation => "MongoDBOperation",
            ErrorKind::MongoDBVersion => "MongoDBVersion",
            ErrorKind::RecordNotFound(_, _) => "RecordNotFound",
        };
        Some(name)
    }
}

/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;
