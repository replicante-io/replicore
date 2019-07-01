extern crate replicante_models_api;

use std::collections::BTreeMap;

use replicante_models_api::HealthStatus;

/// Generic health check for a component.
pub trait HealthCheck: Send + Sync {
    /// Execute the status check.
    fn check(&self) -> HealthStatus;
}

impl<CheckFn> HealthCheck for CheckFn
where
    CheckFn: Fn() -> HealthStatus + Send + Sync + 'static,
{
    fn check(&self) -> HealthStatus {
        self()
    }
}

/// Generic healh check manager and register.
#[derive(Default)]
pub struct HealthChecks {
    checks: BTreeMap<String, Box<dyn HealthCheck>>,
}

impl HealthChecks {
    pub fn new() -> HealthChecks {
        let checks = BTreeMap::new();
        HealthChecks { checks }
    }

    /// Register a named health check.
    ///
    /// Check names are exposed to replicante operators and should be meaningful for them.
    ///
    /// If a check with the given name already exists it will be replaced with the new check.
    pub fn register<C, S>(&mut self, name: S, check: C)
    where
        C: HealthCheck + 'static,
        S: Into<String>,
    {
        self.checks.insert(name.into(), Box::new(check));
    }

    /// Run all the register checks and report the results.
    pub fn run(&self) -> HealthResults {
        let mut results = HealthResults::new();
        for (name, check) in self.checks.iter() {
            let result = check.check();
            results.insert(name.to_string(), result);
        }
        results
    }
}

/// Alias type to a map of health check results.
pub type HealthResults = BTreeMap<String, HealthStatus>;
