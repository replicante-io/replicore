use dirs::home_dir;

use crate::ErrorKind;
use crate::Result;

/// Resole an optional leading `~/` to the current user's HOME path.
pub fn resolve_home(path: &str) -> Result<String> {
    let resolved = if path.starts_with("~/") {
        let home = match home_dir() {
            None => return Err(ErrorKind::HomeNotFound.into()),
            Some(home) => home
                .to_str()
                .expect("user's home to be UTF-8 encoded")
                .to_string(),
        };
        path.replacen('~', &home, 1)
    } else {
        path.to_string()
    };
    Ok(resolved)
}
