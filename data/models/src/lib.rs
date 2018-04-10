extern crate serde;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
extern crate serde_json;

extern crate replicante_agent_models;


mod cluster;
mod node;

pub use self::cluster::Cluster;
pub use self::node::Node;


// WebUI models are not part of the public interface.
pub mod webui;
