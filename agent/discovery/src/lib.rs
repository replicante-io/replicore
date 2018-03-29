#[macro_use]
extern crate error_chain;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;


mod config;
mod engines;
mod errors;

pub use self::config::Config;
pub use self::engines::Iter;
pub use self::engines::discover;
pub use self::errors::*;


/// Inforation about a discovered agent.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Discovery {
    cluster: String,
    target: String,
}

impl Discovery {
    pub fn new<S1, S2>(cluster: S1, target: S2) -> Discovery
        where S1: Into<String>,
              S2: Into<String>
    {
        Discovery {
            cluster: cluster.into(),
            target: target.into(),
        }
    }

    /// ID of the cluster the node belongs to.
    pub fn cluster(&self) -> &String {
        &self.cluster
    }

    /// Address to access the agent at.
    pub fn target(&self) -> &String {
        &self.target
    }
}


#[cfg(test)]
mod tests {
    use super::Discovery;

    #[test]
    fn discovery_item() {
        let discovery = Discovery::new("A", String::from("B"));
        assert_eq!(discovery.cluster(), "A");
        assert_eq!(discovery.target(), "B");
    }
}
