use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use failure::ResultExt;
use opentracingrust::SpanContext;
use opentracingrust::StartOptions;
use opentracingrust::Tracer;
use serde_json;
use sha2::Digest;
use sha2::Sha256;
use slog::debug;
use slog::error;
use slog::info;
use slog::trace;
use slog::warn;
use slog::Logger;

use zookeeper::Acl;
use zookeeper::CreateMode;
use zookeeper::Stat;
use zookeeper::Watcher;
use zookeeper::ZkError;
use zookeeper::ZkResult;
use zookeeper::ZkState;
use zookeeper::ZooKeeper;

use replicante_util_tracing::fail_span;

use super::super::super::config::ZookeeperConfig;
use super::super::super::ErrorKind;
use super::super::super::NodeId;
use super::super::super::Result;
use super::constants::PREFIX_ELECTION;
use super::constants::PREFIX_LOCK;
use super::constants::PREFIX_NODE;
use super::metrics::ZOO_CONNECTION_COUNT;
use super::metrics::ZOO_OP_DURATION;
use super::metrics::ZOO_OP_ERRORS_COUNT;
use super::metrics::ZOO_TIMEOUTS_COUNT;

const HASH_MIN_LEGTH: usize = 4;

/// Wrapper around a `ZooKeeper` instance to handle [re]connection requests.
pub struct Client {
    config: ZookeeperConfig,
    keeper: Option<Mutex<CurrentClient>>,
    logger: Logger,
    registry: Option<RegistryData>,
}

impl Client {
    pub fn new(
        config: ZookeeperConfig,
        node_id: Option<&NodeId>,
        logger: Logger,
    ) -> Result<Client> {
        let registry = match node_id {
            None => None,
            Some(node_id) => {
                let data =
                    serde_json::to_vec(node_id).with_context(|_| ErrorKind::Encode("node id"))?;
                let key = Client::path_from_hash(PREFIX_NODE, &node_id.to_string());
                Some(RegistryData { data, key })
            }
        };
        let mut client = Client {
            config,
            keeper: None,
            logger,
            registry,
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
        Path::new(path)
            .parent()
            .expect("path to Client::container_path must have a parent")
            .to_str()
            .expect("path to Client::container_path must be UTF8")
            .to_string()
    }

    /// Wrapper for `ZooKeeper::create` to track metrics.
    pub fn create(
        keeper: &ZooKeeper,
        path: &str,
        payload: Vec<u8>,
        acl: Vec<Acl>,
        mode: CreateMode,
        span: Option<SpanContext>,
        tracer: Option<&Tracer>,
    ) -> ZkResult<String> {
        let span = match (tracer, span) {
            (Some(tracer), Some(context)) => {
                let options = StartOptions::default().child_of(context);
                let mut span = tracer.span_with_options("coordinator.zookeeper.create", options);
                span.tag("path", path.to_string());
                Some(span.auto_finish())
            }
            _ => None,
        };
        let _timer = ZOO_OP_DURATION.with_label_values(&["create"]).start_timer();
        keeper.create(path, payload, acl, mode).map_err(|error| {
            ZOO_OP_ERRORS_COUNT.with_label_values(&["create"]).inc();
            if error == ZkError::OperationTimeout {
                ZOO_TIMEOUTS_COUNT.inc();
            }
            match span {
                None => error,
                Some(mut span) => {
                    // Existing nodes are not an error for most create operations.
                    if error == ZkError::NodeExists {
                        span.tag("znode.exists", true);
                        error
                    } else {
                        fail_span(error, &mut span)
                    }
                }
            }
        })
    }

    /// Wrapper for `ZooKeeper::delete` to track metrics.
    pub fn delete(
        keeper: &ZooKeeper,
        path: &str,
        version: Option<i32>,
        span: Option<SpanContext>,
        tracer: Option<&Tracer>,
    ) -> ZkResult<()> {
        let span = match (tracer, span) {
            (Some(tracer), Some(context)) => {
                let options = StartOptions::default().child_of(context);
                let mut span = tracer.span_with_options("coordinator.zookeeper.delete", options);
                span.tag("path", path.to_string());
                if let Some(version) = version {
                    span.tag("version", version.to_string());
                }
                Some(span.auto_finish())
            }
            _ => None,
        };
        let _timer = ZOO_OP_DURATION.with_label_values(&["delete"]).start_timer();
        keeper.delete(path, version).map_err(|error| {
            ZOO_OP_ERRORS_COUNT.with_label_values(&["delete"]).inc();
            if error == ZkError::OperationTimeout {
                ZOO_TIMEOUTS_COUNT.inc();
            }
            match span {
                None => error,
                Some(mut span) => {
                    // Missing nodes are not an error for most delete operations.
                    if error == ZkError::NoNode {
                        span.tag("znode.exists", false);
                        error
                    } else {
                        fail_span(error, &mut span)
                    }
                }
            }
        })
    }

    /// Wrapper for `ZooKeeper::exists` to track metrics.
    pub fn exists(
        keeper: &ZooKeeper,
        path: &str,
        watch: bool,
        span: Option<SpanContext>,
        tracer: Option<&Tracer>,
    ) -> ZkResult<Option<Stat>> {
        let span = match (tracer, span) {
            (Some(tracer), Some(context)) => {
                let options = StartOptions::default().child_of(context);
                let mut span = tracer.span_with_options("coordinator.zookeeper.exists", options);
                span.tag("path", path.to_string());
                span.tag("watch", watch);
                Some(span.auto_finish())
            }
            _ => None,
        };
        let _timer = ZOO_OP_DURATION.with_label_values(&["exists"]).start_timer();
        keeper.exists(path, watch).map_err(|error| {
            ZOO_OP_ERRORS_COUNT.with_label_values(&["exists"]).inc();
            if error == ZkError::OperationTimeout {
                ZOO_TIMEOUTS_COUNT.inc();
            }
            match span {
                None => error,
                Some(mut span) => fail_span(error, &mut span),
            }
        })
    }

    /// Wrapper for `ZooKeeper::exists_w` to track metrics.
    pub fn exists_w<W>(
        keeper: &ZooKeeper,
        path: &str,
        watcher: W,
        span: Option<SpanContext>,
        tracer: Option<&Tracer>,
    ) -> ZkResult<Option<Stat>>
    where
        W: Watcher + 'static,
    {
        let span = match (tracer, span) {
            (Some(tracer), Some(context)) => {
                let options = StartOptions::default().child_of(context);
                let mut span = tracer.span_with_options("coordinator.zookeeper.exists_w", options);
                span.tag("path", path.to_string());
                Some(span.auto_finish())
            }
            _ => None,
        };
        let _timer = ZOO_OP_DURATION
            .with_label_values(&["exists_w"])
            .start_timer();
        keeper.exists_w(path, watcher).map_err(|error| {
            ZOO_OP_ERRORS_COUNT.with_label_values(&["exists_w"]).inc();
            if error == ZkError::OperationTimeout {
                ZOO_TIMEOUTS_COUNT.inc();
            }
            match span {
                None => error,
                Some(mut span) => fail_span(error, &mut span),
            }
        })
    }

    /// Wrapper for `ZooKeeper::get_children` to track metrics.
    pub fn get_children(keeper: &ZooKeeper, path: &str, watch: bool) -> ZkResult<Vec<String>> {
        let _timer = ZOO_OP_DURATION
            .with_label_values(&["get_children"])
            .start_timer();
        keeper.get_children(path, watch).map_err(|error| {
            ZOO_OP_ERRORS_COUNT
                .with_label_values(&["get_children"])
                .inc();
            if error == ZkError::OperationTimeout {
                ZOO_TIMEOUTS_COUNT.inc();
            }
            error
        })
    }

    /// Wrapper for `ZooKeeper::get_children_w` to track metrics.
    pub fn get_children_w<W>(keeper: &ZooKeeper, path: &str, watcher: W) -> ZkResult<Vec<String>>
    where
        W: Watcher + 'static,
    {
        let _timer = ZOO_OP_DURATION
            .with_label_values(&["get_children_w"])
            .start_timer();
        keeper.get_children_w(path, watcher).map_err(|error| {
            ZOO_OP_ERRORS_COUNT
                .with_label_values(&["get_children_w"])
                .inc();
            if error == ZkError::OperationTimeout {
                ZOO_TIMEOUTS_COUNT.inc();
            }
            error
        })
    }

    /// Wrapper for `ZooKeeper::get_data` to track metrics.
    pub fn get_data(
        keeper: &ZooKeeper,
        path: &str,
        watch: bool,
        span: Option<SpanContext>,
        tracer: Option<&Tracer>,
    ) -> ZkResult<(Vec<u8>, Stat)> {
        let span = match (tracer, span) {
            (Some(tracer), Some(context)) => {
                let options = StartOptions::default().child_of(context);
                let mut span = tracer.span_with_options("coordinator.zookeeper.get_data", options);
                span.tag("path", path.to_string());
                span.tag("watch", watch);
                Some(span.auto_finish())
            }
            _ => None,
        };
        let _timer = ZOO_OP_DURATION
            .with_label_values(&["get_data"])
            .start_timer();
        keeper.get_data(path, watch).map_err(|error| {
            ZOO_OP_ERRORS_COUNT.with_label_values(&["get_data"]).inc();
            if error == ZkError::OperationTimeout {
                ZOO_TIMEOUTS_COUNT.inc();
            }
            match span {
                None => error,
                Some(mut span) => fail_span(error, &mut span),
            }
        })
    }

    /// Hash a key to return a unique, escaped, identifier.
    pub fn hash_from_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.input(key);
        let hash = hasher.result();
        format!("{:x}", hash)
    }

    /// Create the given path as an empty persistent node.
    ///
    /// `CreateMode::Container` requires Zookeeper 3.5.3+ and does not seem to be reliable
    /// (or maybe I failed to configure the server).
    pub fn mkcontaner(
        keeper: &ZooKeeper,
        path: &str,
        span: Option<SpanContext>,
        tracer: Option<&Tracer>,
    ) -> Result<()> {
        let result = Client::create(
            keeper,
            path,
            Vec::new(),
            Acl::open_unsafe().clone(),
            CreateMode::Persistent,
            span,
            tracer,
        );
        match result {
            Ok(_) => (),
            Err(ZkError::NodeExists) => (),
            Err(error) => {
                return Err(error).with_context(|_| ErrorKind::Backend("container creation"))?;
            }
        };
        Ok(())
    }

    /// Return the full path for the given key.
    ///
    /// The key is hashed first to ensure uniform distribution of keys.
    /// The hashing is also useful to avoid having to deal with string escaping.
    ///
    /// # Panics
    /// If the given hash is not at least `HASH_MIN_LEGTH` characters.
    pub fn path_from_key(root: &str, key: &str) -> String {
        let hash = Client::hash_from_key(key);
        Client::path_from_hash(root, &hash)
    }

    /// Return the full path for the given hashed key.
    ///
    /// # Panics
    /// If the given hash is not at least `HASH_MIN_LEGTH` characters.
    pub fn path_from_hash(root: &str, hash: &str) -> String {
        if hash.len() < HASH_MIN_LEGTH {
            panic!(
                "Client::path_from_hash hash must have at least {} characters",
                HASH_MIN_LEGTH
            );
        }
        let prefix: String = hash.chars().take(HASH_MIN_LEGTH).collect();
        format!("{}/{}/{}", root, prefix, hash)
    }
}

impl Client {
    /// Register the current node ID and attributes.
    fn register_node(keeper: &ZooKeeper, registry: &RegistryData, logger: &Logger) -> Result<()> {
        match Client::register_node_data(keeper, registry) {
            Err(ZkError::NoNode) => {
                let path = Client::container_path(&registry.key);
                Client::mkcontaner(keeper, &path, None, None)?;
                Client::register_node_data(keeper, registry)
                    .with_context(|_| ErrorKind::Backend("node registration"))?;
            }
            Err(err) => {
                return Err(err).with_context(|_| ErrorKind::Backend("node registration"))?;
            }
            Ok(()) => (),
        };
        debug!(logger, "Registered node for debugging");
        Ok(())
    }

    /// Write the data node to zookeeper.
    fn register_node_data(keeper: &ZooKeeper, registry: &RegistryData) -> ZkResult<()> {
        let data = registry.data.clone();
        let key = &registry.key;
        let _timer = ZOO_OP_DURATION.with_label_values(&["create"]).start_timer();
        keeper
            .create(key, data, Acl::read_unsafe().clone(), CreateMode::Ephemeral)
            .map_err(|error| {
                ZOO_OP_ERRORS_COUNT.with_label_values(&["create"]).inc();
                if error == ZkError::OperationTimeout {
                    ZOO_TIMEOUTS_COUNT.inc();
                }
                error
            })?;
        Ok(())
    }
}

impl Client {
    /// Return the current or a new zookeeper client.
    pub fn get(&self) -> Result<Arc<ZooKeeper>> {
        let mutex = self
            .keeper
            .as_ref()
            .expect("current client must be set after creation");
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
        let timer = ZOO_OP_DURATION.with_label_values(&["exists"]).start_timer();
        let not_exists = keeper
            .exists(path, false)
            .map_err(|error| {
                ZOO_OP_ERRORS_COUNT.with_label_values(&["exists"]).inc();
                if error == ZkError::OperationTimeout {
                    ZOO_TIMEOUTS_COUNT.inc();
                }
                error
            })
            .with_context(|_| ErrorKind::Backend("path check"))?
            .is_none();
        timer.observe_duration();
        if not_exists {
            info!(self.logger, "Need to create persistent path"; "path" => path);
            let timer = ZOO_OP_DURATION.with_label_values(&["create"]).start_timer();
            let result = keeper.create(
                path,
                Vec::new(),
                Acl::open_unsafe().clone(),
                CreateMode::Persistent,
            );
            timer.observe_duration();
            match result {
                Ok(_) => (),
                Err(ZkError::NodeExists) => (),
                Err(error) => {
                    ZOO_OP_ERRORS_COUNT.with_label_values(&["create"]).inc();
                    if error == ZkError::OperationTimeout {
                        ZOO_TIMEOUTS_COUNT.inc();
                    }
                    return Err(error).with_context(|_| ErrorKind::Backend("path creation"))?;
                }
            };
        }
        Ok(())
    }

    /// Return a new Zookeeper client that will clear itself when disconnected.
    fn new_client(&self) -> Result<CurrentClient> {
        info!(self.logger, "Initiating new zookeeper session");
        let timeout = Duration::from_secs(self.config.timeout);
        ZOO_CONNECTION_COUNT.inc();
        let timer = ZOO_OP_DURATION
            .with_label_values(&["connect"])
            .start_timer();
        let keeper = ZooKeeper::connect(&self.config.ensemble, timeout, |_| {})
            .map_err(|error| {
                ZOO_OP_ERRORS_COUNT.with_label_values(&["connect"]).inc();
                if error == ZkError::OperationTimeout {
                    ZOO_TIMEOUTS_COUNT.inc();
                }
                error
            })
            .with_context(|_| ErrorKind::BackendConnect)?;
        timer.observe_duration();

        // Make root if needed.
        self.ensure_persistent("/", &keeper)
            .with_context(|_| ErrorKind::Backend("ensure root container exists"))?;
        self.ensure_persistent(PREFIX_ELECTION, &keeper)
            .with_context(|_| ErrorKind::Backend("ensure elections container exists"))?;
        self.ensure_persistent(PREFIX_LOCK, &keeper)
            .with_context(|_| ErrorKind::Backend("ensure locks container exists"))?;
        self.ensure_persistent(PREFIX_NODE, &keeper)
            .with_context(|_| ErrorKind::Backend("ensure nodes container exists"))?;

        // Register node_id for debugging (if provided).
        if let Some(registry) = self.registry.as_ref() {
            Client::register_node(&keeper, registry, &self.logger)?;
        }

        // Listen for connection events to close self.
        let logger = self.logger.clone();
        let active = Arc::new(AtomicBool::new(true));
        let notify_close = Arc::clone(&active);
        keeper.add_listener(move |state| {
            let reset = match state {
                ZkState::AuthFailed => {
                    error!(logger, "Zookeeper authentication error");
                    false
                }
                ZkState::Closed => {
                    warn!(logger, "Zookeeper session closed");
                    true
                }
                ZkState::Connected => {
                    info!(logger, "Zookeeper connection successfull");
                    false
                }
                ZkState::ConnectedReadOnly => {
                    warn!(logger, "Zookeeper connection is read-only");
                    false
                }
                ZkState::Connecting => {
                    debug!(logger, "Zookeeper session connecting");
                    false
                }
                event => {
                    trace!(logger, "Ignoring deprecated zookeeper event"; "event" => ?event);
                    false
                }
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

/// Container for node register data.
struct RegistryData {
    data: Vec<u8>,
    key: String,
}

#[cfg(test)]
mod tests {
    use super::Client;
    use super::PREFIX_NODE;

    #[test]
    fn path_for_key() {
        let key = "discovery/some.cluster.id";
        let path = Client::path_from_key(PREFIX_NODE, key);
        assert_eq!(
            path,
            "/nodes/22db/22db2cab3cb041408a0d3137b5563e82570bd26eb237b1c7cb998292a709d10c"
        );
    }

    #[test]
    fn path_for_node_id() {
        let hash = "3bb728ac2ca63e8f3be0d39195b09b76";
        let path = Client::path_from_hash(PREFIX_NODE, hash);
        assert_eq!(path, "/nodes/3bb7/3bb728ac2ca63e8f3be0d39195b09b76");
    }

    #[test]
    #[should_panic(expected = "hash must have at least 4 characters")]
    fn path_too_short() {
        let hash = "3bb";
        Client::path_from_hash(PREFIX_NODE, hash);
    }
}
