use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use super::super::Result;
use super::super::backend::ElectionBehaviour;


/// Election for a single primary with secondaries ready to take over.
pub struct Election(Box<dyn ElectionBehaviour>);

impl Election {
    pub(crate) fn new(inner: Box<dyn ElectionBehaviour>) -> Self {
        Election(inner)
    }
}

impl Election {
    /// Run for election.
    pub fn run(&mut self) -> Result<()> {
        self.0.run()
    }

    /// Check the current election status.
    pub fn status(&self) -> ElectionStatus {
        self.0.status()
    }

    /// Relinquish primary role, if primary, and remove itself from the election.
    pub fn step_down(&mut self) -> Result<()> {
        self.0.step_down()
    }

    /// Watch the election for changes.
    pub fn watch(&self) -> ElectionWatch {
        self.0.watch()
    }
}

impl Drop for Election {
    fn drop(&mut self) {
        self.0.step_down_on_drop()
    }
}


/// Status of a `Election` instance.
#[derive(Clone, Debug)]
pub enum ElectionStatus {
    /// The election instnace is not a candidate for the `Election`.
    NotCandidate,

    /// The election is currently in progress.
    InProgress,

    /// This election instance is the primary for the `Election`.
    Primary,

    /// This election instance is a secondary.
    Secondary,

    /// This election instance was terminated by the coordinator.
    ///
    /// The reason the election instance was terminated (connection lost,
    /// instance de-registered, ...) is stored in the `String` payload.
    Terminated(String),
}

impl ElectionStatus {
    /// Check if the election is a candidate (primary or secondary) or not.
    pub fn is_candidate(&self) -> bool {
        match self {
            ElectionStatus::Primary => true,
            ElectionStatus::Secondary => true,
            _ => false,
        }
    }

    /// Check if the election is primary.
    pub fn is_primary(&self) -> bool {
        match self {
            ElectionStatus::Primary => true,
            _ => false,
        }
    }
}


/// Lightweight election watcher to check the status for changes.
#[derive(Clone)]
pub struct ElectionWatch(Arc<AtomicBool>);

impl ElectionWatch {
    pub(crate) fn new(inner: Arc<AtomicBool>) -> Self {
        ElectionWatch(inner)
    }

    /// Check if the election is (still) the primary.
    pub fn is_primary(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}
