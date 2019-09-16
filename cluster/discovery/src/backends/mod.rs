use replicante_models_core::cluster::ClusterDiscovery;

use crate::config::Config;
use crate::Result;

mod http;

/// Enumerate supported backends to access their iterators.
enum Backend {
    Http(self::http::Iter),

    #[cfg(test)]
    Test(std::vec::IntoIter<Result<ClusterDiscovery>>),
}

impl Iterator for Backend {
    type Item = Result<ClusterDiscovery>;
    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            Backend::Http(ref mut iter) => iter.next(),
            #[cfg(test)]
            Backend::Test(ref mut iter) => iter.next(),
        }
    }
}

/// Iterator over the agent discovery process.
pub struct Iter {
    active: Option<Backend>,
    backends: Vec<Backend>,
}

impl Iter {
    /// Returns an iterator that consumes all the given backends.
    fn new(mut backends: Vec<Backend>) -> Iter {
        backends.reverse();
        Iter {
            active: None,
            backends,
        }
    }

    /// Iterate over the active backend, if any.
    ///
    /// If the active backend has no more items it is discarded an the method returns `None`.
    fn next_active(&mut self) -> Option<Result<ClusterDiscovery>> {
        match self.active.as_mut().unwrap().next() {
            None => {
                self.active = None;
                None
            }
            some => some,
        }
    }

    /// Iterate over the next backend and activate it.
    ///
    /// If there is no other backend the method returns `None`.
    /// If a backend immediately `None` this method proceeds to the next backend.
    fn next_backend(&mut self) -> Option<Result<ClusterDiscovery>> {
        // While there are backends in the list look for one that returns something.
        while let Some(mut backend) = self.backends.pop() {
            match backend.next() {
                None => continue,
                next => {
                    self.active = Some(backend);
                    return next;
                }
            }
        }

        // If there are no more backends return `None`.
        None
    }
}

impl Iterator for Iter {
    type Item = Result<ClusterDiscovery>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.active.is_some() {
            match self.next_active() {
                None => return self.next_backend(),
                some => return some,
            }
        }
        self.next_backend()
    }
}

/// Starts the agent discover process returning and iterator over results.
///
/// The discovery process is directed by the configuration.
/// Backends are configured and added to the iterator to be consumed in turn.
///
/// Some backends may interact with external systems or the hardware while
/// iterating over the results of the discovery process.
/// Because these external systems may fail, the iterator returns a `Result`.
/// In case of error, the error will be returned and the iterator will attempt
/// to move on with the discovery process.
/// Iterators that experience an unrecoverable error should return the error
/// at first and then return `None` to all subsequent iterations.
pub fn discover(config: Config) -> Iter {
    let mut backends: Vec<Backend> = Vec::new();
    for remote in config.http {
        let backend = http::Iter::new(remote);
        backends.push(Backend::Http(backend));
    }
    Iter::new(backends)
}

#[cfg(test)]
mod tests {
    use replicante_models_core::cluster::ClusterDiscovery;

    use super::Backend;
    use super::Iter;

    #[test]
    fn empty() {
        let mut iter = Iter::new(Vec::new());
        assert!(iter.next().is_none());
    }

    #[test]
    fn only_empty_iters() {
        let cluster_a = Backend::Test(Vec::new().into_iter());
        let cluster_b = Backend::Test(Vec::new().into_iter());
        let cluster_c = Backend::Test(Vec::new().into_iter());
        let mut iter = Iter::new(vec![cluster_a, cluster_b, cluster_c]);
        assert!(iter.next().is_none());
    }

    #[test]
    fn with_backend() {
        let test1 = ClusterDiscovery::new(
            "test1",
            vec![
                "http://node1:port/".into(),
                "http://node2:port/".into(),
                "http://node3:port/".into(),
            ],
        );
        let test2 = ClusterDiscovery::new(
            "test2",
            vec!["http://node1:port/".into(), "http://node3:port/".into()],
        );
        let backend = Backend::Test(vec![Ok(test1.clone()), Ok(test2.clone())].into_iter());
        let mut iter = Iter::new(vec![backend]);
        let next = iter.next().unwrap().unwrap();
        assert_eq!(next, test1);
        let next = iter.next().unwrap().unwrap();
        assert_eq!(next, test2);
        assert!(iter.next().is_none());
    }

    #[test]
    fn with_more_backends() {
        let test0 = ClusterDiscovery::new(
            "mongodb-rs",
            vec![
                "http://node1:37017".into(),
                "http://node2:37017".into(),
                "http://node3:37017".into(),
            ],
        );
        let test1 = ClusterDiscovery::new(
            "test1",
            vec![
                "http://node1:port/".into(),
                "http://node2:port/".into(),
                "http://node3:port/".into(),
            ],
        );
        let test2 = ClusterDiscovery::new(
            "test2",
            vec!["http://node1:port/".into(), "http://node3:port/".into()],
        );
        let cluster_a = Backend::Test(vec![Ok(test0.clone())].into_iter());
        let cluster_b = Backend::Test(Vec::new().into_iter());
        let cluster_c = Backend::Test(vec![Ok(test1.clone()), Ok(test2.clone())].into_iter());
        let mut iter = Iter::new(vec![cluster_a, cluster_b, cluster_c]);
        let next = iter.next().unwrap().unwrap();
        assert_eq!(next, test0);
        let next = iter.next().unwrap().unwrap();
        assert_eq!(next, test1);
        let next = iter.next().unwrap().unwrap();
        assert_eq!(next, test2);
        assert!(iter.next().is_none());
    }
}
