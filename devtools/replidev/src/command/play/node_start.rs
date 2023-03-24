use std::fs::File;

use anyhow::Result;
use replisdk_experimental::platform::templates::TemplateContext;
use replisdk_experimental::platform::templates::TemplateLookup;

use crate::conf::Conf;
use crate::platform::node_start;

use super::StartNodeOpt;

pub async fn run(args: &StartNodeOpt, conf: &Conf) -> Result<i32> {
    // CLI provided key information.
    let cluster_id = args
        .cluster_id
        .as_deref()
        .unwrap_or(&args.store)
        .replace('/', "-");

    // Grab additional attributes from CLI.
    let mut attributes = serde_json::Map::new().into();
    for var in &args.vars {
        let data: serde_json::Value = serde_json::from_str(var)?;
        json_patch::merge(&mut attributes, &data);
    }
    for var_file in &args.var_files {
        let data = File::open(&var_file)?;
        let data = serde_json::from_reader(data)?;
        json_patch::merge(&mut attributes, &data);
    }
    let attributes = match attributes {
        serde_json::Value::Object(attributes) => attributes,
        _ => panic!("json_patch::merge of object must return an object"),
    };

    // Lookup and render the template.
    let factory = crate::platform::TemplateLoader::default();
    let templates = TemplateLookup::load_file(factory, "stores/manifest.yaml").await?;
    let template_context = TemplateContext {
        attributes,
        cluster_id: cluster_id.clone(),
        store: args.store.clone(),
        store_version: args.store_version.clone(),
    };
    let template = templates
        .lookup(&template_context)
        .await?
        .ok_or_else(|| anyhow::anyhow!("node template not found"))?;
    let pod = template.render(template_context)?;

    // Start a new node pod.
    let node_id: String = args
        .node_name
        .clone()
        .unwrap_or_else(|| node_start::random_node_id(8));
    let start_node_spec = node_start::StartNodeSpec {
        cluster_id: &cluster_id,
        node_id: &node_id,
        pod,
        project: conf.project.to_string(),
        store: &args.store,
    };
    node_start::start_node(start_node_spec, conf).await?;

    println!(
        "--> Started {} node {} for cluster {}",
        &args.store, node_id, cluster_id
    );
    Ok(0)
}
