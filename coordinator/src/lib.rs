extern crate serde;
#[macro_use]
extern crate serde_derive;

extern crate replicante_util_rndid;


mod config;
mod node_id;

pub use self::config::Config;
pub use self::node_id::NodeId;
