use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
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


const HASH_MIN_LEGTH: usize = 4;


/// Wrapper around a `ZooKeeper` instance to handle [re]connection requests.
pub struct Client {
    config: ZookeeperConfig,
    keeper: Option<Mutex<CurrentClient>>,
    logger: Logger,
    registry_data: Vec<u8>,
    registry_key: String,
}

impl Client {
    pub fn new(config: ZookeeperConfig, node_id: &NodeId, logger: Logger) -> Result<Client> {
        let registry_data: Vec<u8> = serde_json::to_vec(node_id)
            .context(ErrorKind::Encode("node id"))?;
        let registry_key = Client::path_from_hash("/nodes", &node_id.to_string());
        let mut client = Client {
            config,
            keeper: None,
            logger,
            registry_data,
            registry_key,
        };
        let keeper = Mutex::new(client.new_client()?);
        client.keeper = Some(keeper);
        Ok(client)
    }

    /// Return the path to the container of the given path.
    ///
    /// # Panics
    /// If the given path does not have a container or it is not UTF8.
    ///
    /// # Example
    /// ```ignore
    /// let path = "/a/b/c";
    /// let container = Client::container_path(path);
    /// assert_eq!(container, "/a/b");
    /// ```
    pub fn container_path(path: &str) -> String {
        let path = Path::new(path);
        let path = path.parent().expect("path to Client::container_path must have a parent");
        path.to_str().expect("path to Client::container_path must be UTF8").to_string()
    }

    /// Create the given path as an empty persistent node.
    ///
    /// `CreateMode::Container` requires Zookeeper 3.5.3+ and does not seem to be reliable
    /// (or maybe I failed to configure the server).
    pub fn mkcontaner(keeper: &ZooKeeper, path: &str) -> Result<()> {
        match keeper.create(path, Vec::new(), Acl::open_unsafe().clone(), CreateMode::Persistent) {
            Err(ZkError::NodeExists) => (),
            Err(error) => Err(error).context(ErrorKind::Backend("container creation"))?,
            Ok(_) => (),
        };
        Ok(())
    }

    /// Return the full path for the given hashed key.
    ///
    /// # Panics
    /// If the given hash is not at least `HASH_MIN_LEGTH` characters.
    pub fn path_from_hash(root: &str, hash: &str) -> String {
        if hash.len() < HASH_MIN_LEGTH {
            panic!("Client::path_from_hash hash must have at least {} characters", HASH_MIN_LEGTH);
        }
        let prefix: String = hash.chars().take(HASH_MIN_LEGTH).collect();
        format!("{}/{}/{}", root, prefix, hash)
    }
}

impl Client {
    /// Return the current or a new zookeeper client.
    pub fn get(&self) -> Result<Arc<ZooKeeper>> {
        let mutex = self.keeper.as_ref().expect("current client must be set after creation");
        let mut current = mutex.lock().expect("zookeeper client lock was poisoned");
        if !current.active() {
            let new_client = self.new_client()?;
            *current = new_client;
        }
        Ok(current.client())
    }
}

impl Client {
    /// Ensure the given path exists and create it if it does not.
    fn ensure_persistent(&self, path: &str, keeper: &ZooKeeper) -> Result<()> {
        if keeper.exists(path, false).context(ErrorKind::Backend("path check"))?.is_none() {
            info!(self.logger, "Need to create persistent path"; "path" => path);
            let result = keeper.create(
                path, Vec::new(), Acl::open_unsafe().clone(), CreateMode::Persistent
            );
            match result {
                Ok(_) => (),
                Err(ZkError::NodeExists) => (),
                Err(err) => return Err(err).context(ErrorKind::Backend("path creation"))?,
            };
        }
        Ok(())
    }

    /// Return a new Zookeeper client that will clear itself when disconnected.
    fn new_client(&self) -> Result<CurrentClient> {
        info!(self.logger, "Initiating new zookeeper session");
        let timeout = Duration::from_secs(self.config.timeout);
        let keeper = ZooKeeper::connect(&self.config.ensamble, timeout, |_| {})
            .context(ErrorKind::BackendConnect)?;

        // Make root if needed.
        self.ensure_persistent("/", &keeper).context(ErrorKind::Backend("ensure '/' exists"))?;
        self.ensure_persistent("/nodes", &keeper)
            .context(ErrorKind::Backend("ensure '/nodes' exists"))?;
        self.ensure_persistent("/tombstones", &keeper)
            .context(ErrorKind::Backend("ensure '/tombstones' exists"))?;

        // Register node_id for debugging.
        match self.register_node(&keeper) {
            Err(ZkError::NoNode) => {
                let path = Client::container_path(&self.registry_key);
                Client::mkcontaner(&keeper, &path)?;
                self.register_node(&keeper).context(ErrorKind::Backend("node registration"))?;
            },
            Err(err) => {
                return Err(err).context(ErrorKind::Backend("node registration"))?;
            },
            Ok(_) => (),
        };
        debug!(self.logger, "Registered node for debugging");

        // Listen for connection events to close self.
        let logger = self.logger.clone();
        let active = Arc::new(AtomicBool::new(true));
        let notify_close = Arc::clone(&active);
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
                notify_close.store(false, Ordering::Relaxed);
                debug!(logger, "Zookeeper session marked as not active");
            }
        });

        // Return a client along with its active flag.
        Ok(CurrentClient {
            active,
            keeper: Arc::new(keeper),
        })
    }

    /// Register the current node ID and attributes.
    fn register_node(&self, keeper: &ZooKeeper) -> ZkResult<()> {
        let data = self.registry_data.clone();
        keeper.create(
            &self.registry_key, data, Acl::read_unsafe().clone(), CreateMode::Ephemeral
        )?;
        Ok(())
    }
}


/// Holder of the current zookeeper client with its `active` flag.
struct CurrentClient {
    active: Arc<AtomicBool>,
    keeper: Arc<ZooKeeper>,
}

impl CurrentClient {
    fn active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    fn client(&self) -> Arc<ZooKeeper> {
        Arc::clone(&self.keeper)
    }
}
