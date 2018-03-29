use super::Discovery;
use super::Result;
use super::config::Config;

mod file;


/// Enumerate supported engine iterators.
enum Engine {
    File(self::file::Iter),
}

impl Iterator for Engine {
    type Item = Result<Discovery>;
    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            Engine::File(ref mut iter) => iter.next(),
        }
    }
}


/// Iterator over the agent discovery process.
pub struct Iter {
    active: Option<Engine>,
    engines: Vec<Engine>,
}

impl Iter {
    /// Returns an iterator that consumes all the given engines.
    fn new(mut engines: Vec<Engine>) -> Iter {
        engines.reverse();
        Iter {
            active: None,
            engines
        }
    }

    /// Iterate over the active engine, if any.
    ///
    /// If the active engine has no more items it is discarded an the method returns `None`.
    fn next_active(&mut self) -> Option<Result<Discovery>> {
        match self.active.as_mut().unwrap().next() {
            None => {
                self.active = None;
                None
            },
            some => some
        }
    }

    /// Iterate over the next engine and activate it.
    ///
    /// If there is no other engine the method returns `None`.
    fn next_engine(&mut self) -> Option<Result<Discovery>> {
        match self.engines.pop() {
            None => None,
            Some(mut engine) => {
                let next = engine.next();
                self.active = Some(engine);
                next
            }
        }
    }
}

impl Iterator for Iter {
    type Item = Result<Discovery>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.active.is_some() {
            match self.next_active() {
                None => return self.next_engine(),
                some => return some
            }
        }
        self.next_engine()
    }
}


/// Starts the agent discover process returning and iterator over results.
///
/// The discovery process is directed by the configuration.
/// Engines are configured and added to the iterator to be consumed in turn.
///
/// Some engines may interact with external systems or the hardware while
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
///     let agents = discover(config)?;
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
pub fn discover(config: Config) -> Result<Iter> {
    let mut engines: Vec<Engine> = Vec::new();
    for file in config.files {
        let engine = file::Iter::from_file(file)?;
        engines.push(Engine::File(engine));
    }
    Ok(Iter::new(engines))
}


#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::Discovery;
    use super::Engine;
    use super::Iter;
    use super::file;

    #[test]
    fn empty() {
        let mut iter = Iter::new(Vec::new());
        assert!(iter.next().is_none());
    }

    #[test]
    fn with_engine() {
        let cursor = Cursor::new("cluster: c\ntargets: ['a', 'b', 'c']");
        let engine = file::Iter::from_yaml(cursor).unwrap();
        let mut iter = Iter::new(vec![Engine::File(engine)]);
        assert_eq!(iter.next().unwrap().unwrap(), Discovery::new("c", "a"));
        assert_eq!(iter.next().unwrap().unwrap(), Discovery::new("c", "b"));
        assert_eq!(iter.next().unwrap().unwrap(), Discovery::new("c", "c"));
        assert!(iter.next().is_none());
    }

    #[test]
    fn with_two_engine() {
        let cursor = Cursor::new("cluster: a\ntargets: ['a', 'b', 'c']");
        let cluster_a = file::Iter::from_yaml(cursor).unwrap();
        let cursor = Cursor::new("cluster: b\ntargets: ['d', 'e', 'f']");
        let cluster_b = file::Iter::from_yaml(cursor).unwrap();
        let mut iter = Iter::new(vec![Engine::File(cluster_a), Engine::File(cluster_b)]);
        assert_eq!(iter.next().unwrap().unwrap(), Discovery::new("a", "a"));
        assert_eq!(iter.next().unwrap().unwrap(), Discovery::new("a", "b"));
        assert_eq!(iter.next().unwrap().unwrap(), Discovery::new("a", "c"));
        assert_eq!(iter.next().unwrap().unwrap(), Discovery::new("b", "d"));
        assert_eq!(iter.next().unwrap().unwrap(), Discovery::new("b", "e"));
        assert_eq!(iter.next().unwrap().unwrap(), Discovery::new("b", "f"));
        assert!(iter.next().is_none());
    }
}
