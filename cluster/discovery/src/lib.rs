use replisdk::platform::models::ClusterDiscovery;

use replicante_models_core::cluster::discovery::DiscoveryBackend;
use replicante_models_core::cluster::discovery::DiscoverySettings;

mod backends;
mod error;
mod metrics;

pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;
pub use self::metrics::register_metrics;

use self::backends::http::Iter as HttpIter;

/// Wrapper backend-specific iterators without exposing implementation details.
enum InnerIter {
    Http(HttpIter),

    #[cfg(any(test, feature = "with_test_support"))]
    Test(std::vec::IntoIter<Result<ClusterDiscovery>>),
}

/// Iterate over clusters returned by a backend.
pub struct Iter {
    inner: InnerIter,
}

impl Iter {
    /// Mock cluster discovery by iterating over the given results.
    #[cfg(any(test, feature = "with_test_support"))]
    pub fn mock(iter: Vec<Result<ClusterDiscovery>>) -> Iter {
        let inner = InnerIter::Test(iter.into_iter());
        Iter { inner }
    }
}

impl Iterator for Iter {
    type Item = Result<ClusterDiscovery>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner {
            InnerIter::Http(ref mut iter) => iter.next(),
            #[cfg(any(test, feature = "with_test_support"))]
            InnerIter::Test(ref mut iter) => iter.next(),
        }
    }
}

/// Fetch cluster records from a discovery backend and iterate over them.
pub fn discover(settings: DiscoverySettings) -> Iter {
    let inner = match settings.backend {
        DiscoveryBackend::Http(config) => InnerIter::Http(HttpIter::new(config)),
    };
    Iter { inner }
}
