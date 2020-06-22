use anyhow::Result;

/// Resovle an optional leading `~/` to the current user's HOME path.
pub fn resolve_home(path: &str) -> Result<String> {
    if path.starts_with("~/") {
        let home = home_dir()?;
        Ok(path.replacen('~', &home, 1))
    } else {
        Ok(path.to_string())
    }
}

/// Return the path to the current user home directory.
///
/// This helper is added to replace the now deprecated `dirs`.
///
/// Implement simple variable lookup for linux.
/// Other OS are not currently supported.
fn home_dir() -> Result<String> {
    match std::env::var("HOME") {
        Err(std::env::VarError::NotPresent) => anyhow::bail!("unable to resolve a path $HOME"),
        Err(std::env::VarError::NotUnicode(_)) => anyhow::bail!("unable to UTF-8 decode $HOME"),
        Ok(path) => Ok(path),
    }
}
