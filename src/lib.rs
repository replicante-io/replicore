#[macro_use]
extern crate error_chain;


pub mod errors;
pub use self::errors::{Error, ErrorKind, ResultExt, Result};


#[doc(hidden)]
pub fn run() -> Result<()> {
    println!("Main crate entrypoint");
    Ok(())
}
