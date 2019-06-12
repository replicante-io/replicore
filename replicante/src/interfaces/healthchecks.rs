use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;
use std::time::Instant;

use failure::ResultExt;
use humthreads::Builder;

use replicante_models_api::HealthStatus;
use replicante_service_healthcheck::HealthChecks as HealthChecksRegister;
use replicante_service_healthcheck::HealthResults;
use replicante_util_upkeep::Upkeep;

use super::super::metrics::HEALTHCHECK_DEGRADED;
use super::super::metrics::HEALTHCHECK_FAILED;
use super::super::metrics::HEALTHCHECK_HEALTHY;
use super::super::ErrorKind;
use super::super::Result;

/// Wrapper for a HealthChecks register that periodically runs checks and exposes results.
pub struct HealthChecks {
    cache: HealthResultsCache,
    delay: Option<Duration>,
    register: Option<HealthChecksRegister>,
}

impl HealthChecks {
    pub fn new(delay: Duration) -> HealthChecks {
        let cache = HealthResultsCache::new();
        let delay = Some(delay);
        let register = Some(HealthChecksRegister::new());
        HealthChecks {
            cache,
            delay,
            register,
        }
    }

    /// Access the wrapped HealthChecks register.
    pub fn register(&mut self) -> &mut HealthChecksRegister {
        self.register
            .as_mut()
            .expect("HealthChecks::register() called after HealthChecks::run()")
    }

    /// Return a proxy to cached results.
    ///
    /// These results are automatically updated by a background thread and cached.
    /// The returned object can be used to inspect the latest available health check results.
    pub fn results_proxy(&self) -> HealthResultsCache {
        self.cache.clone()
    }

    /// Start the background thread that periodically runs checks.
    pub fn run(&mut self, upkeep: &mut Upkeep) -> Result<()> {
        let register = match self.register.take() {
            Some(register) => register,
            None => return Err(ErrorKind::InterfaceAlreadyRunning("healthchecks").into()),
        };
        let cache = self.cache.clone();
        let delay = self.delay.take().unwrap();
        let thread = Builder::new("r:i:healthchecks")
            .full_name("replicante:interface:healthchecks")
            .spawn(move |scope| {
                let mut last_check = Instant::now() - (delay * 2);
                while !scope.should_shutdown() {
                    if last_check.elapsed() > delay {
                        let _activity = scope.scoped_activity("running healthchecks");
                        let results = register.run();
                        for (name, result) in results.iter() {
                            HEALTHCHECK_DEGRADED.with_label_values(&[name]).set(0.0);
                            HEALTHCHECK_FAILED.with_label_values(&[name]).set(0.0);
                            HEALTHCHECK_HEALTHY.with_label_values(&[name]).set(0.0);
                            match &result {
                                HealthStatus::Degraded(_) => {
                                    HEALTHCHECK_DEGRADED.with_label_values(&[name]).set(1.0)
                                }
                                HealthStatus::Failed(_) => {
                                    HEALTHCHECK_FAILED.with_label_values(&[name]).set(1.0)
                                }
                                HealthStatus::Healthy => {
                                    HEALTHCHECK_HEALTHY.with_label_values(&[name]).set(1.0)
                                }
                            };
                        }
                        cache.set(results);
                        last_check = Instant::now();
                    }
                    std::thread::sleep(Duration::from_secs(1));
                }
            })
            .with_context(|_| ErrorKind::ThreadSpawn("healthchecks"))?;
        upkeep.register_thread(thread);
        Ok(())
    }
}

/// Internally mutable container for cached check results.
#[derive(Clone)]
pub struct HealthResultsCache(Arc<RwLock<(Instant, HealthResults)>>);

impl HealthResultsCache {
    fn new() -> HealthResultsCache {
        let now = Instant::now();
        let results = HealthResults::new();
        HealthResultsCache(Arc::new(RwLock::new((now, results))))
    }

    /// Get the contents of the cache and the time it was updated.
    pub fn get(&self) -> (Instant, HealthResults) {
        self.0
            .read()
            .expect("HealthResultsCache RwLock was poisoned")
            .clone()
    }

    /// Set the contents of the cache and the time of update.
    fn set(&self, results: HealthResults) {
        let now = Instant::now();
        let mut cache = self
            .0
            .write()
            .expect("HealthResultsCache RwLock was poisoned");
        *cache = (now, results);
    }
}
