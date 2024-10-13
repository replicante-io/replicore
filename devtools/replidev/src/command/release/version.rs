use anyhow::Context;
use anyhow::Result;
use semver::Version;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

/// Read the version from a Cargo.toml file.
pub async fn cargo(path: &str) -> Result<Version> {
    let mut file = File::open(path)
        .await
        .with_context(|| format!("Unable to read version from {}", path))?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)
        .await
        .with_context(|| format!("Unable to read content of {}", path))?;
    let cargo: toml::value::Table =
        toml::from_str(&buffer).with_context(|| format!("Unable to TOML decode {}", path))?;
    let package = cargo
        .get("package")
        .ok_or_else(|| anyhow::anyhow!("Package metadata missing from {}", path))?
        .as_table()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Unsupported type for package metadata, expected table in {}",
                path
            )
        })?;
    let version = package
        .get("version")
        .ok_or_else(|| anyhow::anyhow!("Version attribute not found in {}", path))?
        .as_str()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Unsupported type for version attribute, expected string in {}",
                path
            )
        })?;
    Version::parse(version)
        .with_context(|| format!("Invalid semantic version string '{}'", version))
}

/// Read the version from a package.json file.
pub async fn npm(path: &str) -> Result<Version> {
    let mut file = File::open(path)
        .await
        .with_context(|| format!("Unable to read version from {}", path))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .await
        .with_context(|| format!("Unable to read content of {}", path))?;
    let npm: serde_json::Map<String, serde_json::Value> = serde_json::from_slice(&buffer)
        .with_context(|| format!("Unable to JSON decode {}", path))?;
    let version = npm
        .get("version")
        .ok_or_else(|| anyhow::anyhow!("Version attribute not found in {}", path))?
        .as_str()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Unsupported type for version attribute, expected string in {}",
                path
            )
        })?;
    Version::parse(version)
        .with_context(|| format!("Invalid semantic version string '{}'", version))
}
