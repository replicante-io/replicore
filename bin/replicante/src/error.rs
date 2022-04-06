use std::fmt;

use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use actix_web::ResponseError;
use failure::Backtrace;
use failure::Context;
use failure::Fail;

use replicante_models_core::api::validate::ErrorsCollection;
use replicante_util_failure::SerializableFail;

/// Error information returned by functions in case of errors.
#[derive(Debug)]
pub struct Error(Context<ErrorKind>);

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        self.0.get_context()
    }
}

impl Fail for Error {
    fn backtrace(&self) -> Option<&Backtrace> {
        self.0.backtrace()
    }

    fn cause(&self) -> Option<&dyn Fail> {
        self.0.cause()
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

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        self.kind().http_status()
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        let mut response = HttpResponse::build(status);
        match self.kind() {
            ErrorKind::ValidateFailed(ref errors) => response.json(errors),
            _ => response.json(SerializableFail::from(self)),
        }
    }
}

/// Exhaustive list of possible errors emitted by this crate.
#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "the request body is not valid")]
    APIRequestBodyInvalid,

    #[fail(display = "the request has no body but requires one")]
    APIRequestBodyNotFound,

    #[fail(display = "invalid required request parameter '{}'", _0)]
    APIRequestParameterInvalid(&'static str),

    #[fail(display = "missing required request parameter '{}'", _0)]
    APIRequestParameterNotFound(&'static str),

    #[fail(display = "could not initialise client interface for {}", _0)]
    ClientInit(&'static str),

    #[fail(display = "cloud not generate cluster aggregates")]
    ClusterAggregation,

    #[fail(display = "cloud not refresh cluster state")]
    ClusterRefresh,

    #[fail(display = "could not load configuration")]
    ConfigLoad,

    #[fail(display = "could not coordinate with other processes")]
    Coordination,

    #[fail(display = "could not run already running component '{}'", _0)]
    ComponentAlreadyRunning(&'static str),

    #[fail(display = "error while running the '{}' component", _0)]
    ComponentFailed(&'static str),

    #[fail(display = "could not deserialize {} into {}", _0, _1)]
    Deserialize(&'static str, &'static str),

    #[fail(display = "could not emit a '{}' to the events stream", _0)]
    EventsStreamEmit(&'static str),

    #[fail(display = "could not follow the events stream with group '{}'", _0)]
    EventsStreamFollow(&'static str),

    #[fail(display = "could not run already running interface '{}'", _0)]
    InterfaceAlreadyRunning(&'static str),

    #[fail(display = "could not initialise {} interface", _0)]
    InterfaceInit(&'static str),

    #[fail(display = "could not find model {} with ID {}", _0, _1)]
    ModelNotFound(&'static str, String),

    // TODO(namespace-rollout): Replace this error with more appropriate namespace related errors.
    #[fail(
        display = "only the default namespace is supported at this time, found = {}",
        _0
    )]
    NamespaceRolloutNotDefault(String),

    #[fail(display = "could not delete {} from the primary store", _0)]
    PrimaryStoreDelete(&'static str),

    #[fail(display = "could not query {} from the primary store", _0)]
    PrimaryStoreQuery(&'static str),

    #[fail(display = "could not persist {} to the primary store", _0)]
    PrimaryStorePersist(&'static str),

    #[fail(display = "could not register task worker for queue '{}'", _0)]
    TaskWorkerRegistration(String),

    #[fail(display = "thread terminated with an error")]
    ThreadFailed,

    #[fail(display = "could not spawn new thread for '{}'", _0)]
    ThreadSpawn(&'static str),

    #[fail(display = "validation failed")]
    ValidateFailed(ErrorsCollection),

    #[fail(display = "could not query {} from the view store", _0)]
    ViewStoreQuery(&'static str),

    #[fail(display = "could not persist {} to the view store", _0)]
    ViewStorePersist(&'static str),
}

impl ErrorKind {
    fn http_status(&self) -> StatusCode {
        match self {
            Self::APIRequestBodyInvalid => StatusCode::BAD_REQUEST,
            Self::APIRequestBodyNotFound => StatusCode::BAD_REQUEST,
            Self::APIRequestParameterInvalid(_) => StatusCode::BAD_REQUEST,
            Self::APIRequestParameterNotFound(_) => StatusCode::BAD_REQUEST,
            Self::ModelNotFound(_, _) => StatusCode::NOT_FOUND,
            Self::NamespaceRolloutNotDefault(_) => StatusCode::BAD_REQUEST,
            Self::ValidateFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn kind_name(&self) -> Option<&str> {
        let name = match self {
            Self::APIRequestBodyInvalid => "APIRequestBodyInvalid",
            Self::APIRequestBodyNotFound => "APIRequestBodyNotFound",
            Self::APIRequestParameterInvalid(_) => "APIRequestParameterInvalid",
            Self::APIRequestParameterNotFound(_) => "APIRequestParameterNotFound",
            Self::ClientInit(_) => "ClientInit",
            Self::ClusterAggregation => "ClusterAggregation",
            Self::ClusterRefresh => "ClusterRefresh",
            Self::ConfigLoad => "ConfigLoad",
            Self::Coordination => "Coordination",
            Self::ComponentAlreadyRunning(_) => "ComponentAlreadyRunning",
            Self::ComponentFailed(_) => "ComponentFailed",
            Self::Deserialize(_, _) => "Deserialize",
            Self::EventsStreamEmit(_) => "EventsStreamEmit",
            Self::EventsStreamFollow(_) => "EventsStreamFollow",
            Self::InterfaceAlreadyRunning(_) => "InterfaceAlreadyRunning",
            Self::InterfaceInit(_) => "InterfaceInit",
            Self::ModelNotFound(_, _) => "ModelNotFound",
            Self::NamespaceRolloutNotDefault(_) => "NamespaceRolloutNotDefault",
            Self::PrimaryStoreQuery(_) => "PrimaryStoreQuery",
            Self::PrimaryStoreDelete(_) => "PrimaryStoreDelete",
            Self::PrimaryStorePersist(_) => "PrimaryStorePersist",
            Self::TaskWorkerRegistration(_) => "TaskWorkerRegistration",
            Self::ThreadFailed => "ThreadFailed",
            Self::ThreadSpawn(_) => "ThreadSpawn",
            Self::ValidateFailed(_) => "ValidateFailed",
            Self::ViewStoreQuery(_) => "ViewStoreQuery",
            Self::ViewStorePersist(_) => "ViewStorePersist",
        };
        Some(name)
    }
}

/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;
