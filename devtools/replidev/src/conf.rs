use std::fs::File;
use std::sync::Mutex;

use failure::ResultExt;
use serde::Deserialize;
use serde_yaml::Mapping;
use serde_yaml::Value;

use crate::ErrorKind;
use crate::Result;

const CONF_FILE: &str = "replidev.yaml";
const CONF_FILE_LOCAL: &str = "replidev.local.yaml";

// The first time an IP is detected cache it for consistency and performance.
lazy_static::lazy_static! {
    static ref DETECTED_IP_CACHE: Mutex<Option<String>> = Mutex::new(None);
}

/// Project specific configuration.
#[derive(Debug, Deserialize)]
pub struct Conf {
    /// Command to execute easypki.
    #[serde(default = "Conf::default_easypki")]
    pub easypki: String,

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
    pub fn from_file() -> Result<Self> {
        // Read config file and optional override file.
        let base = Conf::load_file(CONF_FILE)?;
        let local = if std::path::Path::new(CONF_FILE_LOCAL).exists() {
            Conf::load_file(CONF_FILE_LOCAL)?
        } else {
            Mapping::new()
        };

        // Merge the config options and decode the result.
        let conf = Conf::merge(base, local);
        let conf = serde_yaml::from_value(conf).with_context(|_| ErrorKind::ConfigLoad)?;
        Ok(conf)
    }

    /// IP address the `podman-host` alias attached to all pods points to.
    ///
    /// If an IP address is not provided in the configuration an attempt to
    /// auto-detect a non-local IP address is made.
    pub fn podman_host_ip(&self) -> Result<String> {
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
        let error = ErrorKind::ip_not_detected();
        Err(error.into())
    }
}

impl Conf {
    fn default_easypki() -> String {
        "easypki".into()
    }

    fn default_podman() -> String {
        "podman".into()
    }

    fn load_file(file: &str) -> Result<Mapping> {
        let conf = File::open(file).with_context(|_| ErrorKind::ConfigLoad)?;
        let conf = serde_yaml::from_reader(conf).with_context(|_| ErrorKind::ConfigLoad)?;
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
#[derive(PartialEq, Eq, Debug, Deserialize)]
pub enum Project {
    /// Replicante Agents Repository
    #[serde(rename = "agents")]
    Agents,

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
        }
    }
}

impl std::fmt::Display for Project {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Agents => write!(fmt, "agents"),
            Self::Core => write!(fmt, "core"),
            Self::Playground => write!(fmt, "playground"),
        }
    }
}
