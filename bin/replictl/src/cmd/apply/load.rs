//! Logic to load apply manifests from various sources.
use anyhow::Context;
use anyhow::Result;
use futures::Stream;
use futures::StreamExt;
use futures::TryStreamExt;
use serde::Deserialize;
use serde_json::Value as Json;
use tokio::io::AsyncReadExt;

/// Load YAML manifests from sources and stream them when available.
pub fn manifests(sources: Vec<String>) -> impl Stream<Item = Result<Json>> {
    futures::stream::iter(sources)
        // Async read each file and stream the raw data.
        .then(|source| async {
            let err_msg = format!("unable to read '{source}'");
            match source {
                // If the source is - read standard input till the end.
                source if source == "-" => {
                    let mut data = Vec::new();
                    tokio::io::stdin()
                        .read_to_end(&mut data)
                        .await
                        .context(err_msg)?;
                    Ok(data)
                }
                // Otherwise read the entire file.
                source => tokio::fs::read(source).await.context(err_msg),
            }
        })
        // Deserialize read data into a stream of Json documents.
        .map_ok(|payload| {
            let documents = serde_yaml::Deserializer::from_slice(&payload);
            let documents: Vec<Result<Json>> = documents
                .map(|document| {
                    let document = Json::deserialize(document)?;
                    Ok(document)
                })
                .collect();
            futures::stream::iter(documents)
        })
        .try_flatten()
}

#[cfg(test)]
mod tests {
    use futures::TryStreamExt;

    use super::manifests;

    const MANIFEST_DOUBLE: &str = "src/cmd/apply/fixtures/double-manifest.yaml";
    const MANIFEST_SINGLE: &str = "src/cmd/apply/fixtures/single-manifest.yaml";

    #[tokio::test]
    async fn no_sources() {
        let stream = manifests(vec![]);
        let stream: Vec<_> = stream.try_collect().await.expect("manifests to load");
        assert_eq!(stream.len(), 0);
    }

    #[tokio::test]
    async fn source_one_manifest() {
        let stream = manifests(vec![MANIFEST_SINGLE.to_string()]);
        let stream: Vec<_> = stream.try_collect().await.expect("manifests to load");
        assert_eq!(
            stream,
            vec![serde_json::json!({
                "test": "fixture",
            })]
        )
    }

    #[tokio::test]
    async fn source_two_manifest() {
        let stream = manifests(vec![MANIFEST_DOUBLE.to_string()]);
        let stream: Vec<_> = stream.try_collect().await.expect("manifests to load");
        assert_eq!(
            stream,
            vec![
                serde_json::json!({
                    "index": 1,
                }),
                serde_json::json!({
                    "index": 2,
                })
            ]
        )
    }

    #[tokio::test]
    async fn sources_combined() {
        let stream = manifests(vec![
            MANIFEST_SINGLE.to_string(),
            MANIFEST_DOUBLE.to_string(),
        ]);
        let stream: Vec<_> = stream.try_collect().await.expect("manifests to load");
        assert_eq!(
            stream,
            vec![
                serde_json::json!({
                    "test": "fixture",
                }),
                serde_json::json!({
                    "index": 1,
                }),
                serde_json::json!({
                    "index": 2,
                })
            ]
        )
    }
}
