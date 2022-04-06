use std::fs::File;

use anyhow::Context;
use anyhow::Result;
use serde_json::Value;
use slog::debug;
use slog::info;
use slog::Logger;
use structopt::StructOpt;

use replicante_models_core::api::apply::ApplyObject;
use replicante_models_core::api::apply::SCOPE_ATTRS;

use crate::apiclient::RepliClient;
use crate::context::ContextStore;

// Additional apply CLI options.
// NOTE: this is not a docstring because StructOpt then uses it as the actions help.
#[derive(Debug, StructOpt)]
pub struct Opt {
    /// Path to a YAML file to apply or - to read from stdin.
    #[structopt(short, long)]
    pub file: String,
}

/// Execute the selected command.
pub async fn execute(logger: &Logger, opt: &crate::Opt, apply_opt: &Opt) -> Result<i32> {
    // Load and validate the object to apply.
    let object = from_yaml(logger, apply_opt).await?;
    let mut object = ApplyObject::from_raw(object).map_err(crate::InvalidApply::new)?;

    // Apply scope overrides as needed.
    let context = ContextStore::active_context(logger, opt).await?;
    let ns = context.namespace(&opt.context).ok();
    let cluster = context.cluster(&opt.context).ok();
    let node = context.node(&opt.context).ok();
    let scopes = SCOPE_ATTRS.iter().zip(vec![ns, cluster, node]);
    for (scope, value) in scopes {
        let value = match value {
            None => continue,
            // Ignore override if the object expressly defines the scope.
            Some(_) if object.metadata.contains_key(*scope) => continue,
            Some(value) => value,
        };
        debug!(
            logger,
            "Overriding scope value from CLI arguments";
            "scope" => scope,
            "value" => &value,
        );
        let scope = str::to_string(*scope);
        object.metadata.insert(scope, value.into());
    }

    // Send the apply request.
    let client = RepliClient::new(logger, context).await?;
    let response = match client.apply(object).await {
        Ok(response) => response,
        Err(error) => {
            eprintln!("Unable to apply object due to API error");
            return Err(error);
        }
    };
    if let Some(message) = response.get("message") {
        println!("[remote] {}", message);
    }
    println!("Object applied successfully");
    Ok(0)
}

/// YAML-decode the apply object from `FILE` or stdin.
async fn from_yaml(logger: &Logger, opt: &Opt) -> Result<Value> {
    let file = &opt.file;
    info!(logger, "Loading apply data"; "format" => "yaml", "file" => file);
    if file == "-" {
        tokio::task::spawn_blocking(|| {
            let stdin = std::io::stdin();
            serde_yaml::from_reader(stdin)
        })
        .await
        .context("Unable to load stdin to apply")?
        .context("Unable to decode stdin to apply")
    } else {
        let file = file.to_string();
        tokio::task::spawn_blocking(|| -> Result<_> {
            let reader = File::open(file)?;
            let value = serde_yaml::from_reader(reader)?;
            Ok(value)
        })
        .await
        .with_context(|| format!("Unable to load {} to apply", opt.file))?
        .with_context(|| format!("Unable to decode {} to apply", opt.file))
    }
}
