use std::collections::BTreeMap;
use std::fs::File;

use failure::ResultExt;

use crate::conf::Conf;
use crate::podman::Pod;
use crate::settings::paths::Paths;
use crate::ErrorKind;
use crate::Result;

use super::CleanCommonOpt;

static REPLICORE_STACK_NAME: &str = "play-replicore";
static REPLICORE_STACK_FILE: &str = "replicore/stack.yaml";

pub async fn clean(args: &CleanCommonOpt, conf: &Conf) -> Result<i32> {
    let paths = crate::settings::paths::PlayReplicore::new();
    let data = paths.data();
    println!(
        "--> Clean data for {} pod (from {})",
        REPLICORE_STACK_NAME, data
    );
    if args.confirm {
        crate::podman::unshare(conf, vec!["rm", "-r", &data]).await?;
    } else {
        println!("Skipping: you must --confirm deleting data");
    }
    Ok(0)
}

pub async fn start(conf: &Conf) -> Result<i32> {
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
    crate::podman::pod_start(conf, pod.clone(), REPLICORE_STACK_NAME, labels, variables).await?;
    for container in pod.containers {
        if let Some(command) = container.initialise {
            let name = format!("{}-{}", REPLICORE_STACK_NAME, container.name);
            println!(
                "--> Initialise {}/{} from {}",
                REPLICORE_STACK_NAME, container.name, name
            );
            crate::podman::exec(conf, &name, command).await?;
        }
    }
    Ok(0)
}

pub async fn stop(conf: &Conf) -> Result<i32> {
    crate::podman::pod_stop(conf, REPLICORE_STACK_NAME).await?;
    Ok(0)
}
