use std::collections::HashMap;
use std::sync::Mutex;

use super::super::super::config::EventsSnapshotsConfig;
use super::metrics::DISCOVERY_SNAPSHOT_TRACKED_CLUSTERS;


/// Helper object to decide when snapshot events should be emitted.
///
/// This is a naive implementation using an in-memory map that never forgets nodes.
/// As a consequence it can "leak" memory in the presence of frequent cluster rotation.
pub struct EmissionTracker {
    enabled: bool,
    frequency: u32,
    state: Mutex<HashMap<String, u32>>,
}

impl EmissionTracker {
    pub fn new(config: EventsSnapshotsConfig) -> EmissionTracker {
        EmissionTracker {
            enabled: config.enabled,
            frequency: config.frequency,
            state: Mutex::new(HashMap::new()),
        }
    }

    /// Clear the content of the emission tracker.
    pub fn reset(&self) {
        if !self.enabled {
            return;
        }
        let mut map = self.state.lock().expect("EmissionTracker lock was poisoned");
        map.clear();
        DISCOVERY_SNAPSHOT_TRACKED_CLUSTERS.set(0.0);
    }

    /// Determine if it is time to snapshot a cluster.
    pub fn snapshot(&self, cluster: String) -> bool {
        if !self.enabled {
            return false;
        }
        let mut map = self.state.lock().expect("EmissionTracker lock was poisoned");
        // Default to 1 so that we can emit immediatelly.
        // This is so that a failover leads to a double snapshot instead of a snapshot delay.
        let state = map.entry(cluster).or_insert_with(|| {
            DISCOVERY_SNAPSHOT_TRACKED_CLUSTERS.inc();
            1
        });
        *state -= 1;
        if *state == 0 {
            *state = self.frequency;
            return true;
        }
        false
    }
}
