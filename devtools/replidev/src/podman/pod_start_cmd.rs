use std::collections::BTreeMap;
use std::process::Command;

use failure::ResultExt;

use super::Pod;
use crate::settings::Variables;
use crate::Conf;
use crate::ErrorKind;
use crate::Result;

/// Start a pod matching the given definition.
pub fn pod_start<S>(
    conf: &Conf,
    pod: Pod,
    name: S,
    labels: BTreeMap<String, String>,
    variables: Variables,
) -> Result<()>
where
    S: std::fmt::Display,
{
    // Create (but to not start) the pod object.
    println!("--> Create pod {}", name);
    let mut podman = Command::new(&conf.podman);
    podman
        .arg("pod")
        .arg("create")
        .arg(format!("--name={}", name))
        .arg("--add-host")
        .arg(format!("podman-host:{}", conf.podman_host_ip()?));
    let mut labels = labels;
    for port in pod.ports {
        let host_port = port.host;
        let pod_port = port.pod.unwrap_or(port.host);
        if let Some(name) = port.name {
            let label = format!("io.replicante.dev/port/{}", name);
            labels.insert(label, host_port.to_string());
        }
        podman
            .arg("--publish")
            .arg(format!("{}:{}", host_port, pod_port));
    }
    for (key, value) in labels {
        podman.arg("--label").arg(format!("{}={}", key, value));
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
        let mut podman = Command::new(&conf.podman);
        podman
            .arg("run")
            .arg(format!("--pod={}", name))
            .arg(format!("--name={}", con_name))
            .arg("--detach")
            .arg("--init")
            .arg("--tty");
        if let Some(user) = container.user {
            podman.arg("--user").arg(user);
        }
        if let Some(workdir) = container.workdir {
            podman.arg("--workdir").arg(workdir);
        }

        for (key, value) in container.env {
            let value = variables.inject(&value)?;
            podman.arg("--env").arg(format!("{}={}", key, value));
        }

        // Process bind mounts and make missing sources.
        let mut bind_sources = Vec::new();
        for mount in container.mount {
            let mut spec = vec![format!("type={}", mount.mount_type)];
            for (key, value) in mount.options.iter() {
                let value = variables.inject(value)?;
                if mount.mount_type == "bind" && (key == "src" || key == "source") {
                    let value = value.clone();
                    bind_sources.push((mount.clone(), value));
                }
                spec.push(format!("{}={}", key, value));
            }
            let spec = spec.join(",");
            podman.arg("--mount").arg(spec);
        }
        for (mount, source) in bind_sources {
            if !std::path::Path::new(&source).exists() {
                std::fs::create_dir_all(&source)
                    .with_context(|_| ErrorKind::fs_not_allowed(&source))?;
            }
            if let Some(uid) = mount.uid {
                crate::podman::unshare(conf, vec!["chown", &uid.to_string(), &source])?;
            }
        }

        // Append image and command.
        podman.arg(container.image);
        if let Some(command) = container.command {
            for arg in command {
                podman.arg(variables.inject(&arg)?);
            }
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
