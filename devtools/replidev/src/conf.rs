use std::fs::File;
use std::sync::Mutex;

use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde_yaml::Mapping;
use serde_yaml::Value;

const CONF_FILE: &str = "replidev.yaml";
const CONF_FILE_LOCAL: &str = "replidev.local.yaml";
const CONF_LOAD_ERRPR: &str =
    "Could not load configuration, are you in the root of a Replicante repository?";

// The first time an IP is detected cache it for consistency and performance.
lazy_static::lazy_static! {
    static ref DETECTED_IP_CACHE: Mutex<Option<String>> = Mutex::new(None);
}

/// Project specific configuration.
#[derive(Clone, Debug, Deserialize)]
pub struct Conf {
    /// Command to execute easypki.
    #[serde(default = "Conf::default_easypki")]
    pub easypki: String,

    /// List of Cargo.toml files to ignore where looking for crates by replidev release.
    #[serde(default)]
    pub ignored_crates: Vec<String>,

    /// Bind address and port for the playground API server.
    #[serde(default = "Conf::default_play_server_bind")]
    pub play_server_bind: String,

    /// Current project to operate on.
    pub project: Project,

    /// Command to execute podman.
    #[serde(default = "Conf::default_podman")]
    pub podman: String,

    /// IP address the `podman-host` alias attached to all pods points to.
    #[serde(default)]
    pub podman_host_ip: Option<String>,
}

impl Conf {
    /// Load the local project's configuration file.
    pub fn from_file() -> Result<Conf> {
        // Read config file and optional override file.
        let base = Conf::load_file(CONF_FILE)?;
        let local = if std::path::Path::new(CONF_FILE_LOCAL).exists() {
            Conf::load_file(CONF_FILE_LOCAL)?
        } else {
            Mapping::new()
        };

        // Merge the config options and decode the result.
        let conf = Conf::merge(base, local);
        let conf = serde_yaml::from_value(conf).context(CONF_LOAD_ERRPR)?;
        Ok(conf)
    }

    /// IP address the `podman-host` alias attached to all pods points to.
    ///
    /// If an IP address is not provided in the configuration an attempt to
    /// auto-detect a non-local IP address is made.
    pub fn podman_host_ip(&self) -> crate::error::Result<String> {
        // Use configure IP if possible.
        if let Some(ip) = &self.podman_host_ip {
            return Ok(ip.clone());
        }

        // Consult IP cache for consistency and performance.
        {
            let cache = DETECTED_IP_CACHE
                .lock()
                .expect("detected IP cache lock is poisoned");
            if let Some(ip) = cache.as_ref() {
                return Ok(ip.clone());
            }
        }

        // Attempt to auto detect a non-local IP.
        for iface in pnet_datalink::interfaces() {
            for ip in iface.ips {
                let ip = ip.ip();
                if ip.is_loopback() || !ip.is_ipv4() {
                    continue;
                }
                let ip = ip.to_string();
                let mut cache = DETECTED_IP_CACHE
                    .lock()
                    .expect("detected IP cache lock is poisoned");
                *cache = Some(ip.clone());
                return Ok(ip);
            }
        }

        // Could not find a non-loopback IP address.
        let error = crate::error::ErrorKind::ip_not_detected();
        Err(error.into())
        //anyhow::bail!("Could not find a non-loopback IP address");
    }
}

impl Conf {
    fn default_easypki() -> String {
        "easypki".into()
    }

    fn default_play_server_bind() -> String {
        "0.0.0.0:9876".into()
    }

    fn default_podman() -> String {
        "podman".into()
    }

    fn load_file(file: &str) -> Result<Mapping> {
        let conf = File::open(file).context(CONF_LOAD_ERRPR)?;
        let conf = serde_yaml::from_reader(conf).context(CONF_LOAD_ERRPR)?;
        Ok(conf)
    }

    fn merge(mut base: Mapping, local: Mapping) -> Value {
        for (key, value) in local {
            base.insert(key, value);
        }
        Value::Mapping(base)
    }
}

/// Supported replidev projects.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub enum Project {
    /// Replicante Agents Repository
    #[serde(rename = "agents")]
    Agents,

    /// Replicante Common crates for both core and agents.
    #[serde(rename = "common")]
    Common,

    /// Replicante Core
    #[serde(rename = "core")]
    Core,

    /// Replicante Playgrounds Projects
    #[serde(rename = "playground")]
    Playground,
}

impl Project {
    /// Check if a project is allowed to execute the `deps` family of commands.
    pub fn allow_deps(&self) -> bool {
        *self == Self::Core
    }

    /// Check if a project is allowed to execute the `gen-certs` family of commands.
    pub fn allow_gen_certs(&self) -> bool {
        match self {
            Self::Agents => true,
            Self::Core => true,
            Self::Playground => true,
            _ => false,
        }
    }

    /// Check if a project is allowed to execute the `play` family of commands.
    pub fn allow_play(&self) -> bool {
        *self == Self::Playground
    }

    /// Check if a project is allowed to execute the `release` family of commands.
    pub fn allow_release(&self) -> bool {
        match self {
            Self::Agents => true,
            Self::Common => true,
            Self::Core => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for Project {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Agents => write!(fmt, "agents"),
            Self::Common => write!(fmt, "common"),
            Self::Core => write!(fmt, "core"),
            Self::Playground => write!(fmt, "playground"),
        }
    }
}
