use std::collections::BTreeMap;
use std::collections::HashSet;
use std::net::TcpListener;

use anyhow::Context;
use anyhow::Result;
use tokio::process::Command;

use super::Pod;
use crate::podman::Error;
use crate::settings::Variables;
use crate::Conf;

/// Start a pod matching the given definition.
pub async fn pod_start<S>(
    conf: &Conf,
    pod: Pod,
    name: S,
    labels: BTreeMap<String, String>,
    variables: Variables,
) -> Result<()>
where
    S: std::fmt::Display,
{
    // Allocate random ports for any host == 0 definitions.
    let mut pod = pod;
    let mut variables = variables;
    let mut taken: HashSet<usize> = pod
        .ports
        .iter()
        .filter(|port| port.host != 0)
        .map(|port| port.host)
        .collect();
    for mut port in &mut pod.ports {
        if port.host != 0 {
            continue;
        }
        port.host = find_host_port(&taken);
        taken.insert(port.host);
    }
    variables.set_node_name(name.to_string());
    variables.set_ports(&pod.ports);

    // Create (but to not start) the pod object.
    println!("--> Create pod {}", name);
    let mut podman = Command::new(&conf.podman);
    podman
        .arg("pod")
        .arg("create")
        .arg(format!("--name={}", name));

    // Ensure PODMAN_HOSTNAME resolves to slirp4netns virtual router IP.
    if conf.podman_hostname_as_internal {
        let hostname = std::env::var("HOSTNAME").context(Error::InvalidHostnameVar)?;
        podman.arg("--add-host").arg(format!(
            "{}:{}",
            hostname, conf.podman_network_virtual_router_ip
        ));
    }

    // Configure network mode, if set.
    if let Some(network) = &conf.podman_network_mode {
        podman.arg("--network").arg(network);
    }

    // Configure published ports and labels.
    let mut labels = labels;
    for port in pod.ports {
        let host_port = port.host;
        let pod_port = port.pod.unwrap_or(port.host);
        let protocols = port.protocols.unwrap_or_else(|| vec!["tcp".to_string()]);
        for proto in protocols {
            podman
                .arg("--publish")
                .arg(format!("{}:{}/{}", host_port, pod_port, proto));
        }
        if let Some(name) = port.name {
            let label = format!("io.replicante.dev/port/{}", name);
            labels.insert(label, host_port.to_string());
        }
    }
    for (key, value) in labels {
        podman.arg("--label").arg(format!("{}={}", key, value));
    }
    let status = podman.status().await.context(Error::ExecFailed)?;
    if !status.success() {
        let error = Error::CommandFailed(status.code().unwrap_or(-1));
        anyhow::bail!(error);
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
        for (limit, value) in container.ulimit {
            podman.arg("--ulimit").arg(format!("{}={}", limit, value));
        }
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
                std::fs::create_dir_all(&source).with_context(|| Error::io_error(&source))?;
            }
            if let Some(uid) = mount.uid {
                crate::podman::unshare(conf, vec!["chown", &uid.to_string(), &source]).await?;
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
        let status = podman.status().await.context(Error::ExecFailed)?;
        if !status.success() {
            let error = Error::CommandFailed(status.code().unwrap_or(-1));
            anyhow::bail!(error);
        }

        // If the container has a start delay wait a bit.
        if let Some(delay) = container.start_delay {
            println!("--> Waiting {}s for {} to start", delay, con_name);
            let delay = std::time::Duration::from_secs(delay);
            tokio::time::sleep(delay).await;
        }
    }
    Ok(())
}

fn find_host_port(taken: &HashSet<usize>) -> usize {
    let port = (10000..60000).find(|port| {
        if taken.contains(port) {
            return false;
        }
        TcpListener::bind(format!("0.0.0.0:{}", port)).is_ok()
    });
    port.expect("unable to find a usable host port")
}
