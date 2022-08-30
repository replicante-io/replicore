use serde::Deserialize;
use serde::Serialize;

/// Encode an `anyhow::Error` into a fixed structure.
///
/// This structure aims to encode error information into a serialisable type.
/// Errors can then be recorded in DBs or passed over the network as a debugging aid.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// Layer of information about the encoded error.
    pub info: ErrorLayer,

    /// Optional layers of errors that resulted into the encoded error.
    //
    /// Layers are stored closest first so the initial error in the sequence would be last.
    pub causes: Vec<ErrorLayer>,
}

impl From<anyhow::Error> for ErrorInfo {
    fn from(error: anyhow::Error) -> ErrorInfo {
        let mut causes = Vec::new();
        let info = ErrorLayer {
            message: error.to_string(),
        };

        for layer in error.chain().skip(1) {
            causes.push(ErrorLayer {
                message: layer.to_string(),
            })
        }

        ErrorInfo { info, causes }
    }
}

/// Encode information about an exact error which can be part of a chain of errors.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct ErrorLayer {
    /// The error message.
    pub message: String,
    // TODO(backtrace): Add backtraces when support is stable.
    ///// Optional stack backtrace for the error.
    //pub backtrace: Option<String>,
}
