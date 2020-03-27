use std::collections::BTreeMap;
use std::fs::File;

use failure::ResultExt;
use prettytable::cell;
use prettytable::row;
use serde::Deserialize;
use structopt::StructOpt;

use crate::conf::Conf;
use crate::podman::Pod;
use crate::ErrorKind;
use crate::Result;

const PODMAN_DEF_PATH: &str = "devtools/deps/podman";

/// Manage Replicante Core dependencies.
#[derive(Debug, StructOpt)]
pub enum CliOpt {
    /// Delete ALL data store by the given dependencies pods.
    #[structopt(name = "clean")]
    Clean(CleanOpt),

    /// Run the initialise command of each container in the given dependencies pods.
    #[structopt(name = "initialise")]
    Initialise(PodOpt),

    /// List running and available dependencies pods.
    #[structopt(name = "list")]
    List,

    /// Stop and start supported dependencies pods.
    #[structopt(name = "restart")]
    Restart(PodOpt),

    /// Start supported dependencies pods.
    #[structopt(name = "start")]
    Start(PodOpt),

    /// Stop running dependencies pods.
    #[structopt(name = "stop")]
    Stop(PodOpt),
}

#[derive(Debug, StructOpt)]
pub struct CleanOpt {
    /// Confirm deleting the data.
    #[structopt(long)]
    pub confirm: bool,

    #[structopt(flatten)]
    pub pod_opt: PodOpt,
}

#[derive(Debug, StructOpt)]
pub struct PodOpt {
    /// List of pods to start.
    #[structopt(name = "POD", required = true)]
    pods: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PodPsStatus {
    pub id: String,
    pub status: String,
}

/// Manage Replicante Core dependencies.
pub fn run(args: CliOpt, conf: Conf) -> Result<bool> {
    if !conf.project.allow_deps() {
        let error = ErrorKind::invalid_project(conf.project, "replidev deps");
        return Err(error.into());
    }
    match args {
        CliOpt::Clean(args) => clean(&args, &conf),
        CliOpt::Initialise(args) => initialise(&args, &conf),
        CliOpt::List => list(&conf),
        CliOpt::Restart(args) => restart(&args, &conf),
        CliOpt::Start(args) => start(&args, &conf),
        CliOpt::Stop(args) => stop(&args, &conf),
    }
}

fn clean(args: &CleanOpt, conf: &Conf) -> Result<bool> {
    for pod_name in &args.pod_opt.pods {
        let data = format!("./devtools/data/{}", pod_name);
        println!("--> Clean data for {} pod (from {})", pod_name, data);
        if args.confirm {
            crate::podman::unshare(conf, vec!["rm", "-r", &data])?;
        } else {
            println!("Skipping: you must --confirm deleting data");
        }
    }
    Ok(true)
}

fn initialise(args: &PodOpt, conf: &Conf) -> Result<bool> {
    for pod_name in &args.pods {
        let pod = pod_definition(pod_name)?;
        for container in pod.containers {
            if let Some(command) = container.initialise {
                let name = format!("replideps-{}-{}", pod_name, container.name);
                println!(
                    "--> Initialise {}/{} from {}",
                    pod_name, container.name, name
                );
                crate::podman::exec(conf, &name, command)?;
            }
        }
    }
    Ok(true)
}

/// List running and available dependencies pod.
///
/// Example output?
///   NAME      STATUS  POD ID  DEFINITION
///   essential Running abc-123 $PODMAN_DEF_PATH/essential.yaml
///   uis       -       -       $PODMAN_DEF_PATH/uis.yaml
///   legacy    Running def-456 -
fn list(conf: &Conf) -> Result<bool> {
    // Find running dependencies pods.
    let pods = crate::podman::pod_ps(
        conf,
        r#"{{ .Name }}: {id: "{{ .ID }}", status: "{{ .Status }}"}"#,
        vec![
            "label=io.replicante.dev.role=deps",
            &format!("label=io.replicante.dev.project={}", conf.project),
        ],
    )?;
    let pods: BTreeMap<String, PodPsStatus> = if pods.is_empty() {
        BTreeMap::new()
    } else {
        serde_yaml::from_slice(&pods).expect("failed to parse formatted podman pod ps output")
    };
    let pods: BTreeMap<String, PodPsStatus> = pods
        .into_iter()
        .map(|(key, value)| {
            let key = key.trim_start_matches("replideps-").to_string();
            (key, value)
        })
        .collect();

    // Find available replideps definitions.
    let mut available = Vec::new();
    let dir = std::fs::read_dir(PODMAN_DEF_PATH)
        .with_context(|_| ErrorKind::fs_error("failed to list available pods"))?;
    for file in dir {
        let file = file.with_context(|_| ErrorKind::fs_error("failed to list available pods"))?;
        let name = file
            .file_name()
            .into_string()
            .map_err(|_| ErrorKind::fs_error("failed to decode file name"))?;
        if !name.ends_with(".yaml") {
            continue;
        }
        let name = name.trim_end_matches(".yaml").to_string();
        if pods.contains_key(&name) {
            continue;
        }
        available.push(name);
    }
    available.sort();

    // Print a table with the pods and info.
    let mut table = prettytable::Table::new();
    table.add_row(row!["NAME", "STATUS", "POD ID", "DEFINITION"]);
    for pod in pods {
        let name = pod.0;
        let info = pod.1;
        let path = format!("{}/{}.yaml", PODMAN_DEF_PATH, name);
        let def = std::path::Path::new(&path);
        let def = if def.exists() { path } else { "-".to_string() };
        table.add_row(row![name, info.status, info.id, def]);
    }
    for name in available {
        let path = format!("{}/{}.yaml", PODMAN_DEF_PATH, name);
        table.add_row(row![name, "-", "-", path]);
    }

    let format = prettytable::format::FormatBuilder::new()
        .column_separator(' ')
        .padding(0, 3)
        .build();
    table.set_format(format);
    table.printstd();
    Ok(true)
}

/// Helper function to load and decode a pod definition file.
fn pod_definition(name: &str) -> Result<Pod> {
    let definition = format!("{}/{}.yaml", PODMAN_DEF_PATH, name);
    let pod = File::open(definition).with_context(|_| ErrorKind::pod_not_found(name))?;
    let pod = serde_yaml::from_reader(pod).with_context(|_| ErrorKind::invalid_pod(name))?;
    Ok(pod)
}

fn restart(args: &PodOpt, conf: &Conf) -> Result<bool> {
    stop(args, conf)?;
    start(args, conf)
}

fn start(args: &PodOpt, conf: &Conf) -> Result<bool> {
    for pod_name in &args.pods {
        let pod = pod_definition(pod_name)?;
        let variables = crate::podman::Variables::new(pod_name);
        crate::podman::pod_start(
            conf,
            pod,
            format!("replideps-{}", pod_name),
            &conf.project,
            variables,
        )?;
    }
    Ok(true)
}

fn stop(args: &PodOpt, conf: &Conf) -> Result<bool> {
    for pod_name in args.pods.iter().rev() {
        if crate::podman::pod_stop(conf, format!("replideps-{}", pod_name)).is_err() {
            println!(
                "--> Failed to stop {} pod, assuming it was not running",
                pod_name
            );
        }
    }
    Ok(true)
}
