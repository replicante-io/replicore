use std::process::Command;

use failure::ResultExt;

use super::Pod;
use super::Variables;
use crate::ErrorKind;
use crate::Result;

/// Start a pod matching the given definition.
pub fn pod_start<S1, S2>(pod: Pod, name: S1, project: S2, variables: Variables) -> Result<()>
where
    S1: std::fmt::Display,
    S2: std::fmt::Display,
{
    // Create (but to not start) the pod object.
    println!("--> Create pod {}", name);
    let mut podman = Command::new("podman");
    podman
        .arg("pod")
        .arg("create")
        .arg(format!("--name={}", name))
        .arg("--label")
        .arg(format!("io.replicante.dev.project={}", project));
    for port in pod.ports {
        let host_port = port.host;
        let pod_port = port.pod.unwrap_or(port.host);
        podman
            .arg("--publish")
            .arg(format!("{}:{}", host_port, pod_port));
    }
    let status = podman
        .status()
        .with_context(|_| ErrorKind::podman_exec("pod create"))?;
    if !status.success() {
        let error = ErrorKind::podman_failed("pod create");
        return Err(error.into());
    }

    // Start containers in the given order, the first container will also start the pod.
    for container in pod.containers {
        let con_name = format!("{}-{}", name, container.name);
        println!("--> Start container {}", con_name);
        let mut podman = Command::new("podman");
        podman
            .arg("run")
            .arg(format!("--pod={}", name))
            .arg(format!("--name={}", con_name))
            .arg("--detach")
            .arg("--init")
            .arg("--tty");
        for (key, value) in container.env {
            let value = variables.inject(&value);
            podman.arg("--env").arg(format!("{}={}", key, value));
        }

        // Process bind mounts and make missing sources.
        let mut bind_sources = Vec::new();
        for mount in container.mount {
            let mut spec = vec![format!("type={}", mount.mount_type)];
            spec.extend(mount.options.iter().map(|(key, value)| {
                let value = variables.inject(value);
                if mount.mount_type == "bind" && (key == "src" || key == "source") {
                    bind_sources.push(value.clone());
                }
                format!("{}={}", key, value)
            }));
            let spec = spec.join(",");
            podman.arg("--mount").arg(spec);
        }
        for source in bind_sources {
            if !std::path::Path::new(&source).exists() {
                std::fs::create_dir_all(&source)
                    .with_context(|_| ErrorKind::fs_not_allowed(source))?;
            }
        }

        // Append image and command.
        podman.arg(container.image);
        if let Some(command) = container.command {
            podman.args(command);
        }

        // Run the container.
        let status = podman
            .status()
            .with_context(|_| ErrorKind::podman_exec("run"))?;
        if !status.success() {
            let error = ErrorKind::podman_failed("run");
            return Err(error.into());
        }
    }
    Ok(())
}
