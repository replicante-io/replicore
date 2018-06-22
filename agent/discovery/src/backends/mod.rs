use replicante_data_models::ClusterDiscovery;

use super::Result;
use super::config::Config;

mod file;

pub use self::file::DiscoveryFile as DiscoveryFileModel;


/// Enumerate supported backends to access their iterators.
enum Backend {
    File(self::file::Iter),
}

impl Iterator for Backend {
    type Item = Result<ClusterDiscovery>;
    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            Backend::File(ref mut iter) => iter.next(),
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
            },
            some => some
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
                some => return some
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
///
/// # Example
///
/// ```
/// # extern crate error_chain;
/// # extern crate replicante_agent_discovery;
/// #
/// # use replicante_agent_discovery::Config;
/// # use replicante_agent_discovery::Result;
/// # use replicante_agent_discovery::discover;
/// #
/// # fn run() -> Result<()> {
///     let config = Config::default();
///     let agents = discover(config);
///     for agent in agents {
///         let agent = agent?;
///         println!("{:?}", agent);
///     }
/// #     Ok(())
/// # }
/// # fn main() {
/// #     if let Err(ref e) = run() {
/// #         use std::io::Write;
/// #         use error_chain::ChainedError; // trait which holds `display_chain`
/// #         let stderr = &mut ::std::io::stderr();
/// #         let errmsg = "Error writing to stderr";
/// #         writeln!(stderr, "{}", e.display_chain()).expect(errmsg);
/// #         ::std::process::exit(1);
/// #     }
/// # }
/// ```
pub fn discover(config: Config) -> Iter {
    let mut backends: Vec<Backend> = Vec::new();
    for file in config.files {
        let backend = file::Iter::new(file);
        backends.push(Backend::File(backend));
    }
    Iter::new(backends)
}


#[cfg(test)]
mod tests {
    use std::path::Path;
    use replicante_data_models::ClusterDiscovery;

    use super::Backend;
    use super::Iter;
    use super::file;

    /// Helper function to find fixtrue files.
    ///
    /// Neede because `cargo test` and `cargo kcov` behave differently with regards to the working
    /// directory of the executed command (`cargo test` moves to the crate, `cargo kcov` does not).
    pub fn fixture_path(path: &str) -> String {
        let nested = format!("agent/discovery/{}", path);
        if Path::new(&nested).exists() {
            nested
        } else {
            path.to_string()
        }
    }

    #[test]
    fn empty() {
        let mut iter = Iter::new(Vec::new());
        assert!(iter.next().is_none());
    }

    #[test]
    fn only_empty_iters() {
        let cluster_a = file::Iter::new(fixture_path("tests/no.clusters.yaml"));
        let cluster_b = file::Iter::new(fixture_path("tests/no.clusters.yaml"));
        let cluster_c = file::Iter::new(fixture_path("tests/no.clusters.yaml"));
        let mut iter = Iter::new(vec![
            Backend::File(cluster_a), Backend::File(cluster_b), Backend::File(cluster_c)
        ]);
        assert!(iter.next().is_none());
    }

    #[test]
    fn with_backend() {
        let backend = file::Iter::new(fixture_path("tests/two.clusters.yaml"));
        let mut iter = Iter::new(vec![Backend::File(backend)]);
        let next = iter.next().unwrap().unwrap();
        assert_eq!(next, ClusterDiscovery::new("test1", vec![
            "http://node1:port/".into(),
            "http://node2:port/".into(),
            "http://node3:port/".into(),
        ]));
        let next = iter.next().unwrap().unwrap();
        assert_eq!(next, ClusterDiscovery::new("test2", vec![
            "http://node1:port/".into(),
            "http://node3:port/".into(),
        ]));
        assert!(iter.next().is_none());
    }

    #[test]
    fn with_more_backends() {
        let cluster_a = file::Iter::new(fixture_path("file.example.yaml"));
        let cluster_b = file::Iter::new(fixture_path("tests/no.clusters.yaml"));
        let cluster_c = file::Iter::new(fixture_path("tests/two.clusters.yaml"));
        let mut iter = Iter::new(vec![
            Backend::File(cluster_a), Backend::File(cluster_b), Backend::File(cluster_c)
        ]);
        let next = iter.next().unwrap().unwrap();
        assert_eq!(next, ClusterDiscovery::new("mongodb-rs", vec![
            "http://node1:37017".into(),
            "http://node2:37017".into(),
            "http://node3:37017".into(),
        ]));
        let next = iter.next().unwrap().unwrap();
        assert_eq!(next, ClusterDiscovery::new("test1", vec![
            "http://node1:port/".into(),
            "http://node2:port/".into(),
            "http://node3:port/".into(),
        ]));
        let next = iter.next().unwrap().unwrap();
        assert_eq!(next, ClusterDiscovery::new("test2", vec![
            "http://node1:port/".into(),
            "http://node3:port/".into(),
        ]));
        assert!(iter.next().is_none());
    }
}
