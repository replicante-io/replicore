use std::fs::File;

use failure::ResultExt;
use structopt::StructOpt;

use crate::conf::Conf;
use crate::conf::Project;
use crate::podman::Pod;
use crate::ErrorKind;
use crate::Result;

/// Manage Replicante Core dependencies.
#[derive(Debug, StructOpt)]
pub enum CliOpt {
    /// Delete ALL data store by the given dependencies pods.
    #[structopt(name = "clean")]
    Clean(CleanOpt),

    /// Run the initialise command of each container in the given dependencies pods.
    #[structopt(name = "initialise")]
    Initialise(PodOpt),

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

/// Manage Replicante Core dependencies.
pub fn run(args: CliOpt, conf: Conf) -> Result<bool> {
    if conf.project != Project::Core {
        let error = ErrorKind::invalid_project(conf.project, "replidev deps");
        return Err(error.into());
    }
    match args {
        CliOpt::Clean(args) => clean(&args, &conf),
        CliOpt::Initialise(args) => initialise(&args, &conf),
        CliOpt::Restart(args) => restart(&args, &conf),
        CliOpt::Start(args) => start(&args, &conf),
        CliOpt::Stop(args) => stop(&args, &conf),
    }
}

fn clean(args: &CleanOpt, _: &Conf) -> Result<bool> {
    for pod_name in &args.pod_opt.pods {
        let data = format!("./devtools/data/{}", pod_name);
        println!("--> Clean data for {} pod (from {})", pod_name, data);
        if args.confirm {
            crate::podman::unshare(vec!["rm", "-r", &data])?;
        } else {
            println!("Skipping: you must --confirm deleting data");
        }
    }
    Ok(true)
}

fn initialise(args: &PodOpt, _: &Conf) -> Result<bool> {
    for pod_name in &args.pods {
        let pod = pod_definition(pod_name)?;
        for container in pod.containers {
            if let Some(command) = container.initialise {
                let name = format!("replideps-{}-{}", pod_name, container.name);
                println!(
                    "--> Initialise {}/{} from {}",
                    pod_name, container.name, name
                );
                crate::podman::exec(&name, command)?;
            }
        }
    }
    Ok(true)
}

/// Helper function to load and decode a pod definition file.
fn pod_definition(name: &str) -> Result<Pod> {
    let definition = format!("devtools/podman/{}.yaml", name);
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
            pod,
            format!("replideps-{}", pod_name),
            &conf.project,
            variables,
        )?;
    }
    Ok(true)
}

fn stop(args: &PodOpt, _: &Conf) -> Result<bool> {
    for pod_name in args.pods.iter().rev() {
        if crate::podman::pod_stop(format!("replideps-{}", pod_name)).is_err() {
            println!(
                "--> Failed to stop {} pod, assuming it was not running",
                pod_name
            );
        }
    }
    Ok(true)
}
