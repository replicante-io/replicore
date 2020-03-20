mod error;

pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;

pub fn run() -> Result<bool> {
    Ok(false)
}
