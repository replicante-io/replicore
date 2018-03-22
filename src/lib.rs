#[macro_use]
extern crate error_chain;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;


mod config;
pub mod errors;

pub use self::errors::{Error, ErrorKind, ResultExt, Result};


#[doc(hidden)]
pub fn run() -> Result<()> {
    println!("Main crate entrypoint");
    Ok(())
}
