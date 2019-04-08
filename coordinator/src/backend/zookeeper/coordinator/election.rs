use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use failure::ResultExt;
use slog::Logger;

use zookeeper::Acl;
use zookeeper::CreateMode;
use zookeeper::Subscription;
use zookeeper::ZkError;
use zookeeper::ZkState;
use zookeeper::ZooKeeper;

use replicante_util_failure::failure_info;

use super::super::super::super::ErrorKind;
use super::super::super::super::NodeId;
use super::super::super::super::Result;
use super::super::super::super::coordinator::ElectionStatus;
use super::super::super::super::coordinator::ElectionWatch;

use super::super::super::super::metrics::ELECTION_DROP_FAIL;
use super::super::super::super::metrics::ELECTION_DROP_TOTAL;
use super::super::super::super::metrics::ELECTION_RUN_FAIL;
use super::super::super::super::metrics::ELECTION_RUN_TOTAL;
use super::super::super::super::metrics::ELECTION_STEPDOWN_FAIL;
use super::super::super::super::metrics::ELECTION_STEPDOWN_TOTAL;
use super::super::super::super::metrics::ELECTION_TERMINATED;

use super::super::super::ElectionBehaviour;
use super::super::ElectionCandidateInfo;
use super::super::ElectionInfo;
use super::super::client::Client;
use super::super::constants::PREFIX_ELECTION;


/// Atomically manage the current election state.
#[derive(Clone)]
struct AtomicState {
    state: Arc<Mutex<ElectionState>>,
    context: ElectionContext
}

impl AtomicState {
    fn new(context: ElectionContext) -> Self {
        let state = Arc::new(Mutex::new(ElectionState {
            candidate_znode: None,
            primary_watcher: Arc::new(AtomicBool::new(false)),
            state: ElectionStateMachine::NotCandidate,
            subscription: None,
            terminate_reason: None,
        }));
        AtomicState {
            context,
            state,
        }
    }
}

impl AtomicState {
    /// Return a copy of the election context.
    fn context(&self) -> &ElectionContext {
        &self.context
    }

    /// Return the current state machinate state.
    fn get(&self) -> ElectionStateMachine {
        let lock = self.state.lock().expect("AtomicState lock poisoned");
        lock.state.clone()
    }

    /// Return the current cancidate znode, if any.
    fn get_candidate(&self) -> Option<String> {
        let lock = self.state.lock().expect("AtomicState lock poisoned");
        lock.candidate_znode.clone()
    }

    /// Update the election state to mark it as a primary.
    ///
    /// Does nothing if the election is in an invalid state.
    fn primary(&self) {
        let logger = &self.context.logger;
        let name = &self.context.name;
        let mut lock = self.state.lock().expect("AtomicState lock poisoned");
        match lock.state {
            ElectionStateMachine::Primary => (),
            ElectionStateMachine::Registered |
            ElectionStateMachine::Secondary => {
                debug!(logger, "Node elected as primary"; "election" => name);
                lock.state = ElectionStateMachine::Primary;
                lock.primary_watcher.store(true, Ordering::Relaxed);
            },
            _ => {
                debug!(
                    logger, "Attempted transition to primary for a terminated election";
                    "election" => name
                );
            },
        };
    }

    /// Transition to the `ElectionStateMachine::Registered` state if no changes occurred.
    fn register(
        &self, expected: ElectionStateMachine, candidate_znode: String, subscription: Subscription
    ) -> Result<()> {
        let mut lock = self.state.lock().expect("AtomicState lock poisoned");
        if expected != lock.state {
            return Err(ErrorKind::ElectionRunning(self.context.name.clone()).into());
        }
        lock.candidate_znode = Some(candidate_znode);
        lock.state = ElectionStateMachine::Registered;
        lock.subscription = Some(subscription);
        lock.primary_watcher.store(false, Ordering::Relaxed);
        Ok(())
    }

    /// Update the election state to mark it as a primary.
    ///
    /// Does nothing if the election is in an invalid state.
    fn secondary(&self) {
        let logger = &self.context.logger;
        let name = &self.context.name;
        let mut lock = self.state.lock().expect("AtomicState lock poisoned");
        lock.primary_watcher.store(false, Ordering::Relaxed);
        match lock.state {
            ElectionStateMachine::Primary => {
                debug!(logger, "Node demoted to secondary"; "election" => name);
                lock.state = ElectionStateMachine::Secondary;
            },
            ElectionStateMachine::Registered => {
                debug!(logger, "Node transitioned to secondary"; "election" => name);
                lock.state = ElectionStateMachine::Secondary;
            },
            ElectionStateMachine::Secondary => (),
            _ => {
                debug!(
                    logger, "Attempted transition to secondary for a terminated election";
                    "election" => name
                );
            },
        };
    }

    /// Step down from the election.
    fn step_down(&self) -> Result<()> {
        let logger = &self.context.logger;
        let name = &self.context.name;
        debug!(logger, "Stepping down from election"; "election" => name);

        let mut lock = self.state.lock().expect("AtomicState lock poisoned");
        let znode = lock.candidate_znode.take();
        let subscription = lock.subscription.take();
        lock.state = ElectionStateMachine::NotCandidate;
        lock.primary_watcher.store(false, Ordering::Relaxed);
        drop(lock);

        // Attempt to release zookeeper resources.
        let keeper = self.context.client.get()?;
        if let Some(subscription) = subscription {
            keeper.remove_listener(subscription);
        }
        if let Some(znode) = znode {
            match Client::delete(&keeper, &znode, None) {
                Ok(()) => (),
                Err(ZkError::NoNode) => (),
                Err(error) => {
                    let error = Err(error).with_context(|_| ErrorKind::Backend("election step down"));
                    return error.map_err(|e| e.into());
                }
            };
        }
        Ok(())
    }

    /// Terminate the election.
    fn terminate<S: Into<String>>(&self, reason: S) {
        let logger = &self.context.logger;
        let name = &self.context.name;
        let reason: String = reason.into();
        ELECTION_TERMINATED.inc();
        debug!(logger, "Terminating election"; "election" => name, "reason" => &reason);

        let mut lock = self.state.lock().expect("AtomicState lock poisoned");
        let znode = lock.candidate_znode.take();
        let subscription = lock.subscription.take();
        lock.state = ElectionStateMachine::Terminated;
        lock.terminate_reason = Some(reason);
        lock.primary_watcher.store(false, Ordering::Relaxed);
        drop(lock);

        // Attempt to release zookeeper resources.
        let keeper = match self.context.client.get() {
            Ok(keeper) => keeper,
            // We can't get a client.
            // The session will have been invalidated and the subscription removed for us.
            Err(error) => {
                error!(
                    logger, "Election terminated without zookeeper session";
                    "election" => &self.context.name, failure_info(&error)
                );
                return;
            },
        };

        // Remove subscription and candidate znode.
        if let Some(subscription) = subscription {
            keeper.remove_listener(subscription);
        }
        if let Some(znode) = znode {
            match Client::delete(&keeper, &znode, None) {
                Ok(()) => (),
                Err(ZkError::NoNode) => (),
                Err(error) => {
                    error!(
                        logger, "Failed to delete candidate znode for election";
                        "election" => &self.context.name, failure_info(&error)
                    );
                }
            };
        }
    }

    /// Extract the election status from the state machine.
    fn to_status(&self) -> ElectionStatus {
        let lock = self.state.lock().expect("AtomicState lock poisoned");
        match lock.state {
            ElectionStateMachine::NotCandidate => ElectionStatus::NotCandidate,
            ElectionStateMachine::Primary => ElectionStatus::Primary,
            ElectionStateMachine::Registered => ElectionStatus::InProgress,
            ElectionStateMachine::Secondary => ElectionStatus::Secondary,
            ElectionStateMachine::Terminated => ElectionStatus::Terminated(
                lock.terminate_reason.clone().expect("A terminate_reason must be set")
            ),
        }
    }

    /// Create a watcher for primary status.
    fn watch(&self) -> ElectionWatch {
        let lock = self.state.lock().expect("AtomicState lock poisoned");
        let inner = Arc::clone(&lock.primary_watcher);
        ElectionWatch::new(inner)
    }
}


/// Inner election atomic state.
struct ElectionState {
    candidate_znode: Option<String>,
    primary_watcher: Arc<AtomicBool>,
    state: ElectionStateMachine,
    subscription: Option<Subscription>,
    terminate_reason: Option<String>,
}


/// Stages the election state machine can be in.
#[derive(Clone, Eq, PartialEq)]
enum ElectionStateMachine {
    NotCandidate,
    Primary,
    Registered,
    Secondary,
    Terminated,
}

impl ElectionStateMachine {
    fn can_run(&self) -> bool {
        match self {
            ElectionStateMachine::NotCandidate => true,
            ElectionStateMachine::Terminated => true,
            _ => false
        }
    }

    fn running(&self) -> bool {
        match self {
            ElectionStateMachine::Primary => true,
            ElectionStateMachine::Registered => true,
            ElectionStateMachine::Secondary => true,
            _ => false
        }
    }
}


/// Container struct for data used by elections.
#[derive(Clone)]
struct ElectionContext {
    client: Arc<Client>,
    logger: Logger,
    name: String,
    path_candidate: String,
    path_election: String,
    payload_candidate: ElectionCandidateInfo,
    payload_election: ElectionInfo,
}


/// Zookeeper backed primary-secondaries election.
///
/// This election has a small window where two nodes can be primary at the same time.
/// This happens when a node is no longer primary and a secondary is promoted before
/// the "original" primary realises it needs to stop working.
///
/// # Potential herd effect
/// This implementation is similar to the
/// [official recipie](https://zookeeper.apache.org/doc/r3.4.13/recipes.html#sc_leaderElection)
/// with one difference: this implelementation uses a watcher on the election node instead of
/// watching the next candidate.
///
/// This keeps the implementation simpler and allows extentions like priority (by prefixing
/// nodes) but may lead to scalability issues with large clusters.
///
/// If this is a problem, rework the implementation to use more specific watcher as
/// suggested by the official recipe.
/// If priorities are needed they can still be added with an additional delete of the
/// current master if a node is added with lower priority.
pub struct ZookeeperElection {
    state: AtomicState,
}

impl ZookeeperElection {
    pub fn new(client: Arc<Client>, id: &str, owner: NodeId, logger: Logger) -> Self {
        let name = id.to_string();
        let id = Client::hash_from_key(id);
        let path_candidate = format!("{}/{}/candidate-", PREFIX_ELECTION, id);
        let path_election = format!("{}/{}", PREFIX_ELECTION, id);
        let payload_candidate = ElectionCandidateInfo { owner };
        let payload_election = ElectionInfo { name: name.clone() };
        let context = ElectionContext {
            client,
            logger,
            name,
            path_candidate,
            path_election,
            payload_candidate,
            payload_election,
        };
        let state = AtomicState::new(context);
        ZookeeperElection {
            state,
        }
    }
}

impl ZookeeperElection {
    /// Zookeeper watcher callback.
    ///
    /// Fetch the list of candidates, reinstall the watcher and updates the state.
    fn election_changed(state: &AtomicState) {
        // If the election has ended since the last time we watched it exit now.
        if !state.get().running() {
            return;
        }

        // Grab the zookeeper client if possible.
        let context = state.context();
        let keeper = match context.client.get() {
            Ok(keeper) => keeper,
            Err(error) => {
                error!(
                    context.logger, "Failed to refresh election state";
                    "election" => &context.name, failure_info(&error)
                );
                state.terminate("zookeeper session lost");
                return;
            }
        };

        // Get children and reset the watcher.
        let closure_state = state.clone();
        let result = Client::get_children_w(&keeper, &context.path_election, move |_| {
            ZookeeperElection::election_changed(&closure_state);
        });
        match result {
            Ok(candidates) => {
                // Sort candidates lowest to highest.
                let candidates = {
                    let mut cands = candidates;
                    cands.sort();
                    cands
                };
                let primary = match candidates.get(0) {
                    // There are no candidates in this election.
                    // We must have been deleted.
                    None => {
                        state.terminate("election has no candidates");
                        return;
                    },
                    Some(primary) => format!("{}/{}", state.context.path_election, primary),
                };
                let znode = match state.get_candidate() {
                    Some(znode) => znode,
                    // The election must have been shut down elsewhere.
                    None => {
                        debug!(
                            context.logger, "Not updating election without candidate znode";
                            "election" => &context.name
                        );
                        return;
                    }
                };

                // If the first candidate is us we are primary.
                if znode == primary {
                    state.primary();
                    return;
                }

                // If we are in the candidates list (but not first) we are a secondary.
                let found_myself = candidates.iter().any(|candidate| {
                    let full_path = format!("{}/{}", context.path_election, candidate);
                    full_path == znode
                });
                if found_myself {
                    state.secondary();
                    return;
                }

                // If we are not in the candidates list we were deleted.
                state.terminate("election candidate deleted");
            },

            // The entire election was deleted, transition to terminated state.
            Err(ZkError::NoNode) => state.terminate("election deleted"),

            // Unable to refresh election state, termiate.
            Err(error) => {
                error!(
                    context.logger, "Failed to refresh election state";
                    "election" => &context.name, failure_info(&error)
                );
                state.terminate("election refresh failed")
            }
        };
    }

    /// Zookeeper session closed callback.
    fn session_closed(state: &AtomicState) {
        state.terminate("zookeeper session lost");
    }
}

impl ZookeeperElection {
    fn register(&self, keeper: &ZooKeeper) -> Result<String> {
        let context = self.state.context();
        let payload_candidate = serde_json::to_vec(&context.payload_candidate)
            .with_context(|_| ErrorKind::Encode("election candidate information"))?;
        let result = Client::create(
            &keeper, &context.path_candidate, payload_candidate.clone(),
            Acl::read_unsafe().clone(), CreateMode::EphemeralSequential
        );
        match result {
            Ok(candidate) => Ok(candidate),
            Err(ZkError::NoNode) => {
                // Create the election container and try to register agian.
                let payload_election = serde_json::to_vec(&context.payload_election)
                    .with_context(|_| ErrorKind::Encode("election information"))?;
                let result = Client::create(
                    &keeper, &context.path_election, payload_election,
                    Acl::open_unsafe().clone(), CreateMode::Persistent
                );
                match result {
                    Ok(_) => (),
                    Err(ZkError::NodeExists) => (),
                    Err(error) => {
                        let error = Err(error)
                            .with_context(|_| ErrorKind::Backend("election registration"));
                        return error.map_err(|e| e.into());
                    }
                };
                let znode = Client::create(
                    &keeper, &context.path_candidate, payload_candidate,
                    Acl::read_unsafe().clone(), CreateMode::EphemeralSequential
                ).with_context(|_| ErrorKind::Backend("election registration"))?;
                Ok(znode)
            },
            Err(error) => {
                let error = Err(error).with_context(|_| ErrorKind::Backend("election registration"));
                error.map_err(|e| e.into())
            },
        }
    }
}

impl ElectionBehaviour for ZookeeperElection {
    fn run(&mut self) -> Result<()> {
        let context = self.state.context();
        let state = self.state.get();
        if !state.can_run() {
            return Err(ErrorKind::ElectionRunning(context.name.clone()).into());
        }

        // Register node and attach subscriptions.
        ELECTION_RUN_TOTAL.inc();
        let keeper = context.client.get().map_err(|error| {
            ELECTION_RUN_FAIL.inc();
            error
        })?;
        let candidate_znode = self.register(&keeper).map_err(|error| {
            ELECTION_RUN_FAIL.inc();
            error
        })?;
        let closure_state = self.state.clone();
        let subscription = keeper.add_listener(move |zk_state| {
            if let ZkState::Closed = zk_state {
                ZookeeperElection::session_closed(&closure_state);
            }
        });
        self.state.register(state, candidate_znode.clone(), subscription).map_err(|error| {
            // Delete the candidate_znode if we failed to update the state.
            match Client::delete(&keeper, &candidate_znode, None) {
                Ok(()) => (),
                Err(ZkError::NoNode) => (),
                Err(error) => {
                    error!(
                        self.state.context.logger,
                        "Failed to delete cancidate znode for election in invalid state";
                        "election" => &context.name, failure_info(&error)
                    );
                },
            };
            ELECTION_RUN_FAIL.inc();
            error
        })?;

        // Refresh election state and transition to election results.
        ZookeeperElection::election_changed(&self.state);
        Ok(())
    }

    fn status(&self) -> ElectionStatus {
        self.state.to_status()
    }

    fn step_down(&mut self) -> Result<()> {
        ELECTION_STEPDOWN_TOTAL.inc();
        self.state.step_down().map_err(|error| {
            ELECTION_STEPDOWN_FAIL.inc();
            error
        })
    }

    fn step_down_on_drop(&mut self) {
        ELECTION_DROP_TOTAL.inc();
        if let Err(error) = self.state.step_down() {
            ELECTION_DROP_FAIL.inc();
            error!(
                self.state.context.logger, "Failed to automatically step down election";
                "election" => &self.state.context.name, failure_info(&error)
            );
        }
    }

    fn watch(&self) -> ElectionWatch {
        self.state.watch()
    }
}
