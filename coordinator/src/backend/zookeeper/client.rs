use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use failure::ResultExt;
use serde_json;
use slog::Logger;

use zookeeper::Acl;
use zookeeper::CreateMode;
use zookeeper::ZkError;
use zookeeper::ZkResult;
use zookeeper::ZkState;
use zookeeper::ZooKeeper;

use super::super::super::ErrorKind;
use super::super::super::NodeId;
use super::super::super::Result;
use super::super::super::config::ZookeeperConfig;


/// Wrapper around a `ZooKeeper` instance to handle [re]connection requests.
pub struct Client {
    config: ZookeeperConfig,
    keeper: Arc<Mutex<Option<Arc<ZooKeeper>>>>,
    logger: Logger,
    registry_data: Vec<u8>,
    registry_key: String,
}

impl Client {
    pub fn new(config: ZookeeperConfig, node_id: &NodeId, logger: Logger) -> Result<Client> {
        let registry_data: Vec<u8> = serde_json::to_vec(node_id)
            .context(ErrorKind::Encode("node id"))?;
        let registry_key = format!("/nodes/{}", node_id);
        Ok(Client {
            config,
            keeper: Arc::new(Mutex::new(None)),
            logger,
            registry_data,
            registry_key,
        })
    }

    /// Return the current or a new zookeeper client.
    pub fn get(&self) -> Result<Arc<ZooKeeper>> {
        let mut subscribe = false;
        let mut lock = self.keeper.lock().expect("zookeeper client lock was poisoned");
        if lock.is_none() {
            let client = self.new_client()?;
            *lock = Some(Arc::clone(&client));
            subscribe = true;
        }
        let keeper = Arc::clone(lock.as_ref().unwrap());
        drop(lock);
        if subscribe {
            self.subscribe_state(&keeper);
        }
        Ok(keeper)
    }

    /// Utility method to create all parents in a nested path.
    ///
    /// The path is passed as a slice of segment, the full path of each parent is determined by
    /// appending elements to each other separated by a `/`.
    ///
    /// Empty elements will be ignored.
    pub fn mkpath(&self, keeper: &ZooKeeper, path: &[&str]) -> Result<()> {
        let mut full: String = "".into();
        for element in path.iter().filter(|e| **e != "") {
            full = format!("{}/{}", full, element);
            keeper.create(&full, Vec::new(), Acl::open_unsafe().clone(), CreateMode::Persistent)
                .context(ErrorKind::Backend("path creation"))?;
        }
        Ok(())
    }
}

impl Client {
    /// Return a new Zookeeper client that will clear itself when disconnected.
    fn new_client(&self) -> Result<Arc<ZooKeeper>> {
        info!(self.logger, "Initiating new zookeeper session");
        let timeout = Duration::from_secs(self.config.timeout);
        let keeper = ZooKeeper::connect(&self.config.ensamble, timeout, |_| {})
            .context(ErrorKind::BackendConnect)?;

        // Make root if needed.
        if keeper.exists("/", false).context(ErrorKind::Backend("root check"))?.is_none() {
            info!(self.logger, "Need to create replicante root");
            let result = keeper.create(
                "/", Vec::new(), Acl::open_unsafe().clone(), CreateMode::Persistent
            );
            match result {
                Ok(_) | Err(ZkError::NodeExists) => (),
                Err(err) => return Err(err).context(ErrorKind::Backend("root creation"))?,
            };
        }

        // Register node_id for debugging.
        match self.register_node(&keeper) {
            Err(ZkError::NoNode) => {
                debug!(self.logger, "Need to create registry root");
                self.mkpath(&keeper, &["nodes"])?;
                self.register_node(&keeper).context(ErrorKind::Backend("node registration"))?;
            },
            Err(err) => {
                return Err(err).context(ErrorKind::Backend("node registration"))?;
            },
            Ok(_) => (),
        };
        debug!(self.logger, "Registered node for debugging");

        Ok(Arc::new(keeper))
    }

    /// Register the current node ID and attributes.
    fn register_node(&self, keeper: &ZooKeeper) -> ZkResult<()> {
        let data = self.registry_data.clone();
        keeper.create(
            &self.registry_key, data, Acl::read_unsafe().clone(), CreateMode::Ephemeral
        )?;
        Ok(())
    }

    /// Subscribe to zookeeper event changes & drop the shared connection on close.
    fn subscribe_state(&self, keeper: &ZooKeeper) {
        let logger = self.logger.clone();
        let mutex = Arc::clone(&self.keeper);
        keeper.add_listener(move |state| {
            let reset = match state {
                ZkState::AuthFailed => {
                    error!(logger, "Zookeeper authentication error");
                    false
                },
                ZkState::Closed => {
                    warn!(logger, "Zookeeper session closed");
                    true
                },
                ZkState::Connected => {
                    info!(logger, "Zookeeper connection successfull");
                    false
                },
                ZkState::ConnectedReadOnly => {
                    warn!(logger, "Zookeeper connection is read-only");
                    false
                },
                ZkState::Connecting => {
                    debug!(logger, "Zookeeper session connecting");
                    false
                },
                event => {
                    debug!(logger, "Ignoring deprecated zookeeper event"; "event" => ?event);
                    false
                },
            };
            if reset {
                debug!(logger, "Releasing zookeeper session");
                let mut lock = mutex.lock().expect("zookeeper client lock was poisoned");
                *lock = None;
                debug!(logger, "Zookeeper session dropped");
            }
        });
    }
}
