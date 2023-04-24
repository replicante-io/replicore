use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Context;
use anyhow::Result;
use semver::Version;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use super::version;
use crate::conf::Conf;
use crate::conf::ExtractBinaryMode;
use crate::conf::Image;
use crate::conf::VersionFrom;

const EXTRACT_CONTAINER_NAME: &str = "replidev-extract-binaries";
const TARGET_PATH: &str = "target/prebuilt-binaries";

/// Information about what to extract from an image.
#[derive(Debug)]
struct ExtractEntry {
    mode: ExtractBinaryMode,
    path: String,
    target_name: Option<String>,
}

/// Build container images, reusing caches to speed things up.
pub async fn build_for_check(conf: &Conf) -> Result<()> {
    for image in &conf.images {
        println!("--> Building container image for {} ...", image.name);
        let tags = generate_tags(image).await?;
        crate::podman::build(conf, image, true, tags).await?;
    }
    Ok(())
}

/// Build container images, avoiding caches to ensure everything is clean.
pub async fn build_for_publish(conf: &Conf) -> Result<()> {
    for image in &conf.images {
        println!("--> Building container image for {} ...", image.name);
        let tags = generate_tags(image).await?;
        crate::podman::build(conf, image, false, tags).await?;
    }
    Ok(())
}

/// Extract files or directories from an image.
pub async fn extract_binaries(conf: &Conf, skip_pull: bool) -> Result<()> {
    if conf.extract_binaries.is_empty() {
        return Ok(());
    }

    // Resolve extraction images and group entries by tag.
    let mut groups = BTreeMap::new();
    for binary in &conf.extract_binaries {
        let version = find_version(&binary.version).await?;
        let prefix = format!("{}/{}", binary.registry, binary.repo);
        let tag = format!(
            "{}:v{}.{}.{}",
            prefix, version.major, version.minor, version.patch
        );
        let entry = ExtractEntry {
            mode: binary.extract,
            path: binary.path.clone(),
            target_name: binary.target_name.clone(),
        };
        groups.entry(tag).or_insert_with(Vec::new).push(entry);
    }

    // Ensure the target directory exists and is empty.
    let path = Path::new(TARGET_PATH);
    if path.exists() {
        tokio::fs::remove_dir_all(path).await.with_context(|| {
            format!(
                "Unable to remove binaries target directory at {}",
                TARGET_PATH
            )
        })?;
    }
    tokio::fs::create_dir_all(path).await.with_context(|| {
        format!(
            "Unable to create binaries target directory at {}",
            TARGET_PATH
        )
    })?;

    let mut extracted = Vec::new();
    for (tag, entries) in groups.into_iter() {
        // Create a temporary container to extract entries from.
        println!("--> Starting binaries extraction from image {}", tag);
        if !skip_pull {
            crate::podman::pull(conf, &tag).await?;
        }
        crate::podman::run(
            conf,
            &tag,
            EXTRACT_CONTAINER_NAME,
            &["--rm", "--interactive", "--detach"],
            &["cat"],
        )
        .await?;

        // Extract entries from the container.
        let result = extract_entries(conf, entries).await;

        // Delete the extraction container, even on extraction fail.
        crate::podman::stop(conf, EXTRACT_CONTAINER_NAME).await?;
        extracted.extend(result?);
    }

    // Generate binaries checksum file.
    let mut checksum = Command::new("sha256sum");
    checksum
        .args(extracted)
        .current_dir(TARGET_PATH)
        .stdout(std::process::Stdio::piped());
    let child = checksum
        .spawn()
        .context("Failed to checksum extracted files")?;
    let output = child
        .wait_with_output()
        .await
        .context("Failed to checksum extracted files")?;
    if !output.status.success() {
        anyhow::bail!("Failed to checksum extracted files");
    }
    let path = format!("{}/checksum.txt", TARGET_PATH);
    let mut checksum = tokio::fs::File::create(&path)
        .await
        .with_context(|| format!("Failed to create checksum file at {}", &path))?;
    checksum
        .write_all(&output.stdout)
        .await
        .with_context(|| format!("Failed to write to checksum file at {}", &path))?;
    Ok(())
}

/// Push container images, built in advance, to docker hub.
pub async fn push(conf: &Conf) -> Result<()> {
    for image in &conf.images {
        println!("--> Pushing container image for {} ...", image.name);
        let tags = generate_tags(image).await?;
        for tag in tags {
            crate::podman::push(conf, &tag).await?;
        }
    }
    Ok(())
}

/// Extract every entry from the running extraction container.
async fn extract_entries(conf: &Conf, entries: Vec<ExtractEntry>) -> Result<Vec<String>> {
    let mut extracted = Vec::new();
    for entry in entries {
        let file = match entry.mode {
            ExtractBinaryMode::Directory => {
                extract_directory(conf, &entry.path, entry.target_name).await?
            }
            ExtractBinaryMode::File => extract_file(conf, &entry.path, entry.target_name).await?,
        };
        extracted.push(file);
    }
    Ok(extracted)
}

/// Extract a directory as a tar archive from the running extraction container.
async fn extract_directory(conf: &Conf, path: &str, target_name: Option<String>) -> Result<String> {
    // Figure out the name the archive file will have.
    let file = match target_name {
        Some(file) => file,
        None => Path::new(path)
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Extraction path does not have a file name: {}", path))?
            .to_string_lossy()
            .to_string(),
    };
    let file = format!("{}.tar.gz", file);

    // Create the archive inside the extractor container.
    let archive_path = format!("/home/replicante/{}", file);
    let from = format!("{}:{}", EXTRACT_CONTAINER_NAME, archive_path);
    crate::podman::exec(
        conf,
        EXTRACT_CONTAINER_NAME,
        vec![
            "tar".to_string(),
            "--create".to_string(),
            "--gzip".to_string(),
            "--file".to_string(),
            archive_path,
            "-C".to_string(),
            path.to_string(),
            ".".to_string(),
        ],
    )
    .await
    .with_context(|| format!("Failed to archive {}", path))?;

    // Copy the archive out of the extractor container.
    let to = format!("{}/{}", TARGET_PATH, file);
    crate::podman::copy(conf, &from, &to).await?;
    Ok(file)
}

/// Extract a file from the running extraction container.
async fn extract_file(conf: &Conf, path: &str, target_name: Option<String>) -> Result<String> {
    let file = match target_name {
        Some(file) => file,
        None => Path::new(path)
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Extraction path does not have a file name: {}", path))?
            .to_string_lossy()
            .to_string(),
    };
    let from = format!("{}:{}", EXTRACT_CONTAINER_NAME, path);
    let to = format!("{}/{}", TARGET_PATH, file);
    crate::podman::copy(conf, &from, &to).await?;
    Ok(file)
}

/// Find the image version from a supported source.
async fn find_version(from: &VersionFrom) -> Result<Version> {
    match from {
        VersionFrom::Cargo { path } => version::cargo(path).await,
        VersionFrom::Npm { path } => version::npm(path).await,
    }
}

/// Generate the list of tags to use for the image.
async fn generate_tags(image: &Image) -> Result<Vec<String>> {
    let version = find_version(&image.version).await?;
    let prefix = format!("{}/{}", image.registry, image.repo);
    let tags = vec![
        format!(
            "{}:v{}.{}.{}",
            prefix, version.major, version.minor, version.patch
        ),
        format!("{}:v{}.{}", prefix, version.major, version.minor),
        format!("{}:v{}", prefix, version.major),
        format!("{}:latest", prefix),
    ];
    Ok(tags)
}
