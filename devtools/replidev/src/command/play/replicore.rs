use std::collections::BTreeMap;
use std::fs::File;

use failure::ResultExt;

use super::CleanOpt;
use crate::conf::Conf;
use crate::podman::Pod;
use crate::settings::paths::Paths;
use crate::ErrorKind;
use crate::Result;

static REPLICORE_STACK_NAME: &str = "play-replicore";
static REPLICORE_STACK_FILE: &str = "replicore/stack.yaml";

pub fn clean(args: &CleanOpt, conf: &Conf) -> Result<bool> {
    let paths = crate::settings::paths::PlayReplicore::new();
    let data = paths.data();
    println!(
        "--> Clean data for {} pod (from {})",
        REPLICORE_STACK_NAME, data
    );
    if args.confirm {
        crate::podman::unshare(conf, vec!["rm", "-r", &data])?;
    } else {
        println!("Skipping: you must --confirm deleting data");
    }
    Ok(true)
}

pub fn start(conf: &Conf) -> Result<bool> {
    // Load node definition.
    let def =
        File::open(REPLICORE_STACK_FILE).with_context(|_| ErrorKind::pod_not_found("replicore"))?;
    let pod: Pod =
        serde_yaml::from_reader(def).with_context(|_| ErrorKind::invalid_pod("replicore"))?;

    // Inject cluster & pod name annotations.
    let labels = {
        let mut labels = BTreeMap::new();
        labels.insert(
            "io.replicante.dev/project".to_string(),
            conf.project.to_string(),
        );
        labels
    };

    // Prepare the stack template environment.
    let paths = crate::settings::paths::PlayReplicore::new();
    let variables = crate::settings::Variables::new(conf, paths)?;

    // Start the stack pod and run optional initialisation commands.
    crate::podman::pod_start(conf, pod.clone(), REPLICORE_STACK_NAME, labels, variables)?;
    for container in pod.containers {
        if let Some(command) = container.initialise {
            let name = format!("{}-{}", REPLICORE_STACK_NAME, container.name);
            println!(
                "--> Initialise {}/{} from {}",
                REPLICORE_STACK_NAME, container.name, name
            );
            crate::podman::exec(conf, &name, command)?;
        }
    }
    Ok(true)
}

pub fn stop(conf: &Conf) -> Result<bool> {
    crate::podman::pod_stop(conf, REPLICORE_STACK_NAME)?;
    Ok(true)
}
