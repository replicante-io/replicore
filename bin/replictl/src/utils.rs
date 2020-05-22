use anyhow::Result;
use dirs::home_dir;

/// Resovle an optional leading `~/` to the current user's HOME path.
pub fn resolve_home(path: &str) -> Result<String> {
    if path.starts_with("~/") {
        let home = match home_dir() {
            None => anyhow::bail!("unable to resolve a path $HOME"),
            Some(home) => home
                .to_str()
                .expect("user's home to be UTF-8 encoded")
                .to_string(),
        };
        Ok(path.replacen('~', &home, 1))
    } else {
        Ok(path.to_string())
    }
}
