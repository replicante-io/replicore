#[macro_use]
extern crate error_chain;


pub mod errors;
pub use self::errors::{Error, ErrorKind, ResultExt, Result};
