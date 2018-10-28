use failure::Error;


/// Errors returned by the task system.
#[derive(Debug, Fail)]
pub enum TaskError {
    #[fail(display = "{}", _0)]
    Msg(String),
}


/// Shortcut alias for `Result<T, Error>`;
pub type Result<T> = ::std::result::Result<T, Error>;
