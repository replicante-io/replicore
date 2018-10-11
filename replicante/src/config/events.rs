use replicante_streams_events::Config as EventsStreamConfig;


/// Replicante events configuration options.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct EventsConfig {
    /// Periodic state snapshots configuration.
    #[serde(default)]
    pub snapshots: SnapshotsConfig,

    /// Events streaming backend.
    #[serde(default)]
    pub stream: EventsStreamConfig,
}

impl Default for EventsConfig {
    fn default() -> Self {
        Self {
            snapshots: SnapshotsConfig::default(),
            stream: EventsStreamConfig::default(),
        }
    }
}


/// Periodic state snapshots configuration.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct SnapshotsConfig {
    /// Enables the emission of snapshot events.
    #[serde(default = "SnapshotsConfig::default_enabled")]
    pub enabled: bool,

    /// Frequency (expressed as number of cluster state fetches) of snapshot emission.
    #[serde(default = "SnapshotsConfig::default_frequency")]
    pub frequency: u32,
}

impl Default for SnapshotsConfig {
    fn default() -> Self {
        Self {
            enabled: Self::default_enabled(),
            frequency: Self::default_frequency(),
        }
    }
}

impl SnapshotsConfig {
    /// Default enabled status.
    fn default_enabled() -> bool { true }

    /// Default snapshot emission frequency.
    fn default_frequency() -> u32 { 60 }
}
