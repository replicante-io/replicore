use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use super::super::backend::ElectionBehaviour;
use super::super::Result;

/// Election for a single primary with secondaries ready to take over.
pub struct Election {
    inner: Box<dyn ElectionBehaviour>,
    name: String,
}

impl Election {
    pub(crate) fn new<S>(name: S, inner: Box<dyn ElectionBehaviour>) -> Self
    where
        S: Into<String>,
    {
        Election {
            inner,
            name: name.into(),
        }
    }
}

impl Election {
    /// Name of this election.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Run for election.
    pub fn run(&mut self) -> Result<()> {
        self.inner.run()
    }

    /// Check the current election status.
    pub fn status(&self) -> ElectionStatus {
        self.inner.status()
    }

    /// Relinquish primary role, if primary, and remove itself from the election.
    pub fn step_down(&mut self) -> Result<()> {
        self.inner.step_down()
    }

    /// Watch the election for changes.
    pub fn watch(&self) -> ElectionWatch {
        self.inner.watch()
    }
}

impl Drop for Election {
    fn drop(&mut self) {
        self.inner.step_down_on_drop()
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
        matches!(self, ElectionStatus::Primary | ElectionStatus::Secondary)
    }

    /// Check if the election is primary.
    pub fn is_primary(&self) -> bool {
        matches!(self, ElectionStatus::Primary)
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
