use std::time::Duration;

use crossbeam_channel::bounded;
use crossbeam_channel::Receiver;
use crossbeam_channel::RecvTimeoutError;
use crossbeam_channel::Sender;
use slog::debug;
use slog::info;
use slog::Logger;

use super::super::Error;
use super::super::Result;
use super::Election;
use super::ElectionStatus;

/// Helper class to manage repeating exclusive tasks.
///
/// This main entry point is `LoopingElection::run` and loops until stopped.
/// At each loop, code is executed using "logic hooks" based on the state of the election.
pub struct LoopingElection {
    election: Election,
    election_term: Option<u64>,
    election_term_current: u64,
    logger: Logger,
    logic: Box<dyn LoopingElectionLogic>,
    loop_delay: Duration,
    shutdown_receiver: Option<ShutdownReceiver>,
}

impl LoopingElection {
    pub fn new(options: LoopingElectionOpts, logger: Logger) -> LoopingElection {
        let election_term_current = options.election_term.unwrap_or(0);
        LoopingElection {
            election: options.election,
            election_term: options.election_term,
            election_term_current,
            logger,
            logic: options.logic,
            loop_delay: options.loop_delay,
            shutdown_receiver: options.shutdown_receiver,
        }
    }

    /// Loop continuosly and switches logic according to the current state of the election.
    ///
    /// This method blocks until the election is terminated by one of the logic method
    /// returning `LoopingElectionControl::Exit`.
    pub fn loop_forever(&mut self) {
        let mut first = true;
        loop {
            // After the first loop, susped the thread for a bit to avoid busy looping.
            if !first {
                let loop_delay = self.loop_delay;
                match self.shutdown_receiver.as_ref() {
                    None => ::std::thread::sleep(loop_delay),
                    Some(receiver) => match receiver.recv_timeout(loop_delay) {
                        Ok(()) => break,
                        Err(RecvTimeoutError::Disconnected) => break,
                        Err(RecvTimeoutError::Timeout) => (),
                    },
                };
            }
            first = false;

            // If the term expired, rerun election.
            if self.election_term.is_some() {
                if self.election_term_current == 0 {
                    let flow = self.rerun_election();
                    match flow {
                        LoopingElectionControl::Continue => continue,
                        LoopingElectionControl::Exit => break,
                        LoopingElectionControl::Proceed => (),
                        flow => panic!("unexpected control flow requested: {:?}", flow),
                    }
                }
                self.election_term_current -= 1;
            }

            // Run a single election cycle.
            let flow = self.loop_once();
            match flow {
                LoopingElectionControl::Continue => continue,
                LoopingElectionControl::Exit => break,
                LoopingElectionControl::Proceed => (),
                flow => panic!("unexpected control flow requested: {:?}", flow),
            }
        }

        // Once out of the loop, step down to make sure we are not holding onto an election.
        let flow = self.step_down();
        match flow {
            LoopingElectionControl::Continue => (),
            LoopingElectionControl::Exit => (),
            LoopingElectionControl::Proceed => (),
            flow => panic!("unexpected control flow requested: {:?}", flow),
        };
    }

    /// Run through a cycle based on the election status.
    pub fn loop_once(&mut self) -> LoopingElectionControl {
        // Run logic for cycle start.
        match self.pre_check() {
            LoopingElectionControl::Proceed => (),
            flow => match self.handle_control_flow(flow) {
                LoopingElectionControl::Proceed => (),
                flow => return flow,
            },
        };

        // Run logic based on the election state.
        let status = self.election.status();
        let flow = match status {
            ElectionStatus::NotCandidate => self.not_candidate(),
            ElectionStatus::InProgress => {
                debug!(self.logger, "Election in progress"; "election" => self.election.name());
                LoopingElectionControl::Proceed
            }
            ElectionStatus::Primary => self.primary(),
            ElectionStatus::Secondary => self.secondary(),
            ElectionStatus::Terminated(reason) => self.terminated(reason),
        };
        match flow {
            LoopingElectionControl::Proceed => (),
            flow => match self.handle_control_flow(flow) {
                LoopingElectionControl::Proceed => (),
                flow => return flow,
            },
        };

        // Run logic for cycle end.
        match self.post_check() {
            LoopingElectionControl::Proceed => (),
            flow => match self.handle_control_flow(flow) {
                LoopingElectionControl::Proceed => (),
                flow => return flow,
            },
        };
        LoopingElectionControl::Proceed
    }
}

impl LoopingElection {
    /// Handle control flow requests (like re-runs and step-downs).
    fn handle_control_flow(&mut self, flow: LoopingElectionControl) -> LoopingElectionControl {
        match flow {
            LoopingElectionControl::ReRun => self.rerun_election(),
            LoopingElectionControl::StepDown => self.step_down(),
            _ => flow,
        }
    }

    /// Handle errors encountered while operating.
    fn handle_result(&mut self, result: Result<LoopingElectionControl>) -> LoopingElectionControl {
        let flow = match result {
            Err(error) => self.logic.handle_error(error),
            Ok(flow) => flow,
        };
        self.handle_control_flow(flow)
    }

    /// Called when the election state is `ElectionStatus::NotCandidate`.
    fn not_candidate(&mut self) -> LoopingElectionControl {
        let result = self.logic.not_candidate(&self.election);
        self.handle_result(result)
    }

    /// Called at the end of each cycle.
    fn post_check(&mut self) -> LoopingElectionControl {
        let result = self.logic.post_check(&self.election);
        self.handle_result(result)
    }

    /// Called at the beginning of each cycle.
    fn pre_check(&mut self) -> LoopingElectionControl {
        let result = self.logic.pre_check(&self.election);
        self.handle_result(result)
    }

    /// Called when the election state is `ElectionStatus::Primary`.
    fn primary(&mut self) -> LoopingElectionControl {
        let result = self.logic.primary(&self.election);
        self.handle_result(result)
    }

    /// Run an election by stepping down and trying to get elected again.
    fn rerun_election(&mut self) -> LoopingElectionControl {
        info!(self.logger, "Running for (re-)election"; "election" => self.election.name());
        debug!(self.logger, "Stepping down election"; "election" => self.election.name());
        match self.election.step_down() {
            Ok(()) => (),
            Err(error) => {
                let flow = self.logic.handle_error(error);
                match flow {
                    LoopingElectionControl::Proceed => (),
                    flow => return flow,
                };
            }
        };
        debug!(self.logger, "Running for election"; "election" => self.election.name());
        let flow = match self.election.run() {
            Ok(()) => LoopingElectionControl::Proceed,
            Err(error) => self.logic.handle_error(error),
        };
        self.election_term_current = self.election_term.unwrap_or(0);
        flow
    }

    /// Called when the election state is `ElectionStatus::Secondary`.
    fn secondary(&mut self) -> LoopingElectionControl {
        let result = self.logic.secondary(&self.election);
        self.handle_result(result)
    }

    /// Step down from primary or secondary role.
    fn step_down(&mut self) -> LoopingElectionControl {
        info!(self.logger, "Stepping down election"; "election" => self.election.name());
        match self.election.step_down() {
            Ok(()) => LoopingElectionControl::Proceed,
            Err(error) => self.logic.handle_error(error),
        }
    }

    /// Called when the election state is `ElectionStatus::Terminated`.
    fn terminated(&mut self, reason: String) -> LoopingElectionControl {
        let result = self.logic.terminated(&self.election, reason);
        self.handle_result(result)
    }
}

/// Possible options for logic methods to control the looping election.
#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use]
pub enum LoopingElectionControl {
    /// Continue to the next loop cycle.
    Continue,

    /// Terminate the loop and the election.
    Exit,

    /// Proceed with the normal flow of events.
    Proceed,

    /// Run an election to become primary or secondary.
    ReRun,

    /// Step down from an election, relinquishing primary or secondary role.
    StepDown,
}

/// Implementation of usefull logic through hooks.
pub trait LoopingElectionLogic {
    /// Handle errors encountered while operating.
    fn handle_error(&self, error: Error) -> LoopingElectionControl;

    /// Called when the election state is `ElectionStatus::NotCandidate`.
    ///
    /// By default, trigger a run for election.
    fn not_candidate(&self, _election: &Election) -> Result<LoopingElectionControl> {
        Ok(LoopingElectionControl::ReRun)
    }

    /// Called at the end of each cycle.
    fn post_check(&self, _election: &Election) -> Result<LoopingElectionControl> {
        Ok(LoopingElectionControl::Proceed)
    }

    /// Called at the beginning of each cycle.
    fn pre_check(&self, _election: &Election) -> Result<LoopingElectionControl> {
        Ok(LoopingElectionControl::Proceed)
    }

    /// Called when the election state is `ElectionStatus::Primary`.
    fn primary(&self, election: &Election) -> Result<LoopingElectionControl>;

    /// Called when the election state is `ElectionStatus::Secondary`.
    fn secondary(&self, election: &Election) -> Result<LoopingElectionControl>;

    /// Called when the election state is `ElectionStatus::Terminated`.
    ///
    /// By default, trigger a run for election.
    fn terminated(&self, _election: &Election, _reason: String) -> Result<LoopingElectionControl> {
        Ok(LoopingElectionControl::ReRun)
    }
}

/// Options passed to a `LoopingElection` to customise its behaviour.
pub struct LoopingElectionOpts {
    election: Election,
    election_term: Option<u64>,
    logic: Box<dyn LoopingElectionLogic>,
    loop_delay: Duration,
    shutdown_receiver: Option<ShutdownReceiver>,
}

impl LoopingElectionOpts {
    /// Return a channel to request asynchronous shutdown.
    pub fn shutdown_channel() -> (ShutdownSender, ShutdownReceiver) {
        let (sender, receiver) = bounded(0);
        (ShutdownSender(sender), ShutdownReceiver(receiver))
    }
}

impl LoopingElectionOpts {
    pub fn new<Logic>(election: Election, logic: Logic) -> LoopingElectionOpts
    where
        Logic: LoopingElectionLogic + 'static,
    {
        LoopingElectionOpts {
            election,
            election_term: None,
            logic: Box::new(logic),
            loop_delay: Duration::from_secs(60),
            shutdown_receiver: None,
        }
    }

    /// Remove the election term and never auto-rerun elections.
    pub fn clear_election_term(mut self) -> LoopingElectionOpts {
        self.election_term = None;
        self
    }

    /// Rerun the election after a number of loops.
    ///
    /// When an election term is set, the election is stepped down after the given term.
    ///
    /// A term is defined as a number of loops to execute before stepping down.
    /// The combination of `term` and `loop_delay` can be used to determine an approximate time
    /// interval between election runs.
    ///
    /// The usefulness of and election term is to make failovers part of the norm and not an
    /// occasional, unpredictable event.
    ///
    /// # Panics
    /// The election terms must be at least one.
    /// This method panics if `term` is 0.
    pub fn election_term(mut self, term: u64) -> LoopingElectionOpts {
        if term == 0 {
            panic!("LoopingElectionOpts::election_term requires at least 1 term");
        }
        self.election_term = Some(term);
        self
    }

    /// Set the delay between each loop cycle.
    pub fn loop_delay(mut self, delay: Duration) -> LoopingElectionOpts {
        self.loop_delay = delay;
        self
    }

    /// Set a receiver for a shutdown signal.
    ///
    /// Once a receiver is set, loop delays can be interrupted with a signal or by closing the
    /// sender side of the shurdown channel.
    ///
    /// If a loop delay is interrupted, `LoopingElection::loop_forever` will step down
    /// the election and terminate.
    pub fn shutdown_receiver(mut self, receiver: ShutdownReceiver) -> LoopingElectionOpts {
        self.shutdown_receiver = Some(receiver);
        self
    }
}

/// Type of receivers of shutdown requests for `LoopingElection`.
pub struct ShutdownReceiver(Receiver<()>);

impl ShutdownReceiver {
    /// Wait for a message to be received, the channel to be closed, or the timeout to expire.
    fn recv_timeout(&self, duration: Duration) -> ::std::result::Result<(), RecvTimeoutError> {
        self.0.recv_timeout(duration)
    }
}

/// Type of senders of shutdown requests for `LoopingElection`.
#[derive(Clone)]
pub struct ShutdownSender(Sender<()>);

impl ShutdownSender {
    /// Request the shutdown of the associated `LoopingElection`.
    pub fn request(&self) {
        // Ignore disconnected channels.
        let _ = self.0.send(());
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::time::Duration;

    use slog::o;
    use slog::Discard;
    use slog::Logger;

    use super::super::super::mock::MockCoordinator;
    use super::super::super::Error;
    use super::super::super::Result;
    use super::super::Election;
    use super::super::ElectionStatus;

    use super::LoopingElection;
    use super::LoopingElectionControl;
    use super::LoopingElectionLogic;
    use super::LoopingElectionOpts;

    #[derive(Clone)]
    struct TestLogic {
        pub handle_error: Rc<RefCell<usize>>,
        pub in_progress: Rc<RefCell<usize>>,
        pub max_loops: Rc<RefCell<u32>>,
        pub not_candidate: Rc<RefCell<usize>>,
        pub post_check: Rc<RefCell<usize>>,
        pub pre_check: Rc<RefCell<usize>>,
        pub primary: Rc<RefCell<usize>>,
        pub secondary: Rc<RefCell<usize>>,
        pub step_down: bool,
        pub terminated: Rc<RefCell<usize>>,
    }

    impl TestLogic {
        fn new() -> TestLogic {
            TestLogic {
                handle_error: Rc::new(RefCell::new(0)),
                in_progress: Rc::new(RefCell::new(0)),
                max_loops: Rc::new(RefCell::new(1)),
                not_candidate: Rc::new(RefCell::new(0)),
                post_check: Rc::new(RefCell::new(0)),
                pre_check: Rc::new(RefCell::new(0)),
                primary: Rc::new(RefCell::new(0)),
                secondary: Rc::new(RefCell::new(0)),
                step_down: false,
                terminated: Rc::new(RefCell::new(0)),
            }
        }
    }

    impl LoopingElectionLogic for TestLogic {
        fn handle_error(&self, _: Error) -> LoopingElectionControl {
            *self.handle_error.borrow_mut() += 1;
            LoopingElectionControl::Exit
        }

        fn not_candidate(&self, _: &Election) -> Result<LoopingElectionControl> {
            *self.not_candidate.borrow_mut() += 1;
            Ok(LoopingElectionControl::ReRun)
        }

        fn post_check(&self, _: &Election) -> Result<LoopingElectionControl> {
            *self.post_check.borrow_mut() += 1;
            let mut max_loops = self.max_loops.borrow_mut();
            if *max_loops == 0 {
                return Ok(LoopingElectionControl::Exit);
            }
            *max_loops -= 1;
            Ok(LoopingElectionControl::Proceed)
        }

        fn pre_check(&self, _: &Election) -> Result<LoopingElectionControl> {
            *self.pre_check.borrow_mut() += 1;
            Ok(LoopingElectionControl::Proceed)
        }

        fn primary(&self, _: &Election) -> Result<LoopingElectionControl> {
            *self.primary.borrow_mut() += 1;
            if self.step_down {
                Ok(LoopingElectionControl::StepDown)
            } else {
                Ok(LoopingElectionControl::Proceed)
            }
        }

        fn secondary(&self, _: &Election) -> Result<LoopingElectionControl> {
            *self.secondary.borrow_mut() += 1;
            Ok(LoopingElectionControl::Proceed)
        }

        fn terminated(&self, _: &Election, _: String) -> Result<LoopingElectionControl> {
            *self.terminated.borrow_mut() += 1;
            Ok(LoopingElectionControl::ReRun)
        }
    }

    fn mock_coordinator() -> MockCoordinator {
        let logger = ::slog::Logger::root(::slog::Discard, o!());
        MockCoordinator::new(logger)
    }

    #[test]
    fn loop_once_not_connected() {
        let mock_coordinator = mock_coordinator();
        let mock_election = mock_coordinator.election("test");
        let coordinator = mock_coordinator.mock();
        let election = coordinator.election("test");
        let logic = TestLogic::new();
        let opts = LoopingElectionOpts::new(election, logic.clone());
        let mut looper = LoopingElection::new(opts, Logger::root(Discard, o!()));
        let flow = looper.loop_once();
        {
            let status = mock_election.status.lock().unwrap();
            match *status {
                ElectionStatus::Secondary => (),
                ref status => panic!("unexpected election status, status is {:?}", status),
            }
        }
        assert_eq!(LoopingElectionControl::Proceed, flow);
        assert_eq!(0, *logic.handle_error.borrow());
        assert_eq!(0, *logic.in_progress.borrow());
        assert_eq!(1, *logic.not_candidate.borrow());
        assert_eq!(0, *logic.primary.borrow());
        assert_eq!(0, *logic.secondary.borrow());
        assert_eq!(0, *logic.terminated.borrow());
    }

    #[test]
    fn loop_once_in_progress() {
        let mock_coordinator = mock_coordinator();
        let mock_election = mock_coordinator.election("test");
        {
            let mut status = mock_election.status.lock().unwrap();
            *status = ElectionStatus::InProgress;
        }
        let coordinator = mock_coordinator.mock();
        let election = coordinator.election("test");
        let logic = TestLogic::new();
        let opts = LoopingElectionOpts::new(election, logic.clone());
        let mut looper = LoopingElection::new(opts, Logger::root(Discard, o!()));
        let flow = looper.loop_once();
        {
            let status = mock_election.status.lock().unwrap();
            match *status {
                ElectionStatus::InProgress => (),
                ref status => panic!("unexpected election status, status is {:?}", status),
            }
        }
        assert_eq!(LoopingElectionControl::Proceed, flow);
        assert_eq!(0, *logic.handle_error.borrow());
        assert_eq!(0, *logic.in_progress.borrow());
        assert_eq!(0, *logic.not_candidate.borrow());
        assert_eq!(0, *logic.primary.borrow());
        assert_eq!(0, *logic.secondary.borrow());
        assert_eq!(0, *logic.terminated.borrow());
    }

    #[test]
    fn loop_once_primary() {
        let mock_coordinator = mock_coordinator();
        let mock_election = mock_coordinator.election("test");
        {
            let mut status = mock_election.status.lock().unwrap();
            *status = ElectionStatus::Primary;
        }
        let coordinator = mock_coordinator.mock();
        let election = coordinator.election("test");
        let logic = TestLogic::new();
        let opts = LoopingElectionOpts::new(election, logic.clone());
        let mut looper = LoopingElection::new(opts, Logger::root(Discard, o!()));
        let flow = looper.loop_once();
        {
            let status = mock_election.status.lock().unwrap();
            match *status {
                ElectionStatus::Primary => (),
                ref status => panic!("unexpected election status, status is {:?}", status),
            }
        }
        assert_eq!(LoopingElectionControl::Proceed, flow);
        assert_eq!(0, *logic.handle_error.borrow());
        assert_eq!(0, *logic.in_progress.borrow());
        assert_eq!(0, *logic.not_candidate.borrow());
        assert_eq!(1, *logic.primary.borrow());
        assert_eq!(0, *logic.secondary.borrow());
        assert_eq!(0, *logic.terminated.borrow());
    }

    #[test]
    fn loop_once_secondary() {
        let mock_coordinator = mock_coordinator();
        let mock_election = mock_coordinator.election("test");
        {
            let mut status = mock_election.status.lock().unwrap();
            *status = ElectionStatus::Secondary;
        }
        let coordinator = mock_coordinator.mock();
        let election = coordinator.election("test");
        let logic = TestLogic::new();
        let opts = LoopingElectionOpts::new(election, logic.clone());
        let mut looper = LoopingElection::new(opts, Logger::root(Discard, o!()));
        let flow = looper.loop_once();
        {
            let status = mock_election.status.lock().unwrap();
            match *status {
                ElectionStatus::Secondary => (),
                ref status => panic!("unexpected election status, status is {:?}", status),
            }
        }
        assert_eq!(LoopingElectionControl::Proceed, flow);
        assert_eq!(0, *logic.handle_error.borrow());
        assert_eq!(0, *logic.in_progress.borrow());
        assert_eq!(0, *logic.not_candidate.borrow());
        assert_eq!(0, *logic.primary.borrow());
        assert_eq!(1, *logic.secondary.borrow());
        assert_eq!(0, *logic.terminated.borrow());
    }

    #[test]
    fn loop_once_pre_post_check() {
        let mock_coordinator = mock_coordinator();
        let coordinator = mock_coordinator.mock();
        let election = coordinator.election("test");
        let logic = TestLogic::new();
        let opts = LoopingElectionOpts::new(election, logic.clone());
        let mut looper = LoopingElection::new(opts, Logger::root(Discard, o!()));
        let flow = looper.loop_once();
        assert_eq!(LoopingElectionControl::Proceed, flow);
        assert_eq!(0, *logic.handle_error.borrow());
        assert_eq!(0, *logic.in_progress.borrow());
        assert_eq!(1, *logic.not_candidate.borrow());
        assert_eq!(1, *logic.post_check.borrow());
        assert_eq!(1, *logic.pre_check.borrow());
        assert_eq!(0, *logic.primary.borrow());
        assert_eq!(0, *logic.secondary.borrow());
        assert_eq!(0, *logic.terminated.borrow());
    }

    #[test]
    fn loop_once_step_down() {
        let mock_coordinator = mock_coordinator();
        let mock_election = mock_coordinator.election("test");
        {
            let mut status = mock_election.status.lock().unwrap();
            *status = ElectionStatus::Primary;
        }
        let coordinator = mock_coordinator.mock();
        let election = coordinator.election("test");
        let logic = {
            let mut logic = TestLogic::new();
            logic.step_down = true;
            logic
        };
        let opts = LoopingElectionOpts::new(election, logic.clone());
        let mut looper = LoopingElection::new(opts, Logger::root(Discard, o!()));
        let flow = looper.loop_once();
        {
            let status = mock_election.status.lock().unwrap();
            match *status {
                ElectionStatus::NotCandidate => (),
                ref status => panic!("unexpected election status, status is {:?}", status),
            }
        }
        assert_eq!(LoopingElectionControl::Proceed, flow);
        assert_eq!(0, *logic.handle_error.borrow());
        assert_eq!(0, *logic.in_progress.borrow());
        assert_eq!(0, *logic.not_candidate.borrow());
        assert_eq!(1, *logic.primary.borrow());
        assert_eq!(0, *logic.secondary.borrow());
        assert_eq!(0, *logic.terminated.borrow());
    }

    #[test]
    fn loop_once_terminated() {
        let mock_coordinator = mock_coordinator();
        let mock_election = mock_coordinator.election("test");
        {
            let mut status = mock_election.status.lock().unwrap();
            *status = ElectionStatus::Terminated("test".into());
        }
        let coordinator = mock_coordinator.mock();
        let election = coordinator.election("test");
        let logic = TestLogic::new();
        let opts = LoopingElectionOpts::new(election, logic.clone());
        let mut looper = LoopingElection::new(opts, Logger::root(Discard, o!()));
        let flow = looper.loop_once();
        {
            let status = mock_election.status.lock().unwrap();
            match *status {
                ElectionStatus::Secondary => (),
                ref status => panic!("unexpected election status, status is {:?}", status),
            }
        }
        assert_eq!(LoopingElectionControl::Proceed, flow);
        assert_eq!(0, *logic.handle_error.borrow());
        assert_eq!(0, *logic.in_progress.borrow());
        assert_eq!(0, *logic.not_candidate.borrow());
        assert_eq!(0, *logic.primary.borrow());
        assert_eq!(0, *logic.secondary.borrow());
        assert_eq!(1, *logic.terminated.borrow());
    }

    #[test]
    fn new() {
        let mock_coordinator = mock_coordinator();
        let coordinator = mock_coordinator.mock();
        let election = coordinator.election("test");
        let opts = LoopingElectionOpts::new(election, TestLogic::new());
        let _looper = LoopingElection::new(opts, Logger::root(Discard, o!()));
    }

    #[test]
    fn periodic_rerun() {
        let mock_coordinator = mock_coordinator();
        let mock_election = mock_coordinator.election("test");
        {
            let mut status = mock_election.status.lock().unwrap();
            *status = ElectionStatus::Primary;
        }
        let coordinator = mock_coordinator.mock();
        let election = coordinator.election("test");
        let logic = {
            let logic = TestLogic::new();
            *logic.max_loops.borrow_mut() = 20;
            logic
        };
        let opts = LoopingElectionOpts::new(election, logic.clone())
            .election_term(5)
            .loop_delay(Duration::from_millis(0));
        let mut looper = LoopingElection::new(opts, Logger::root(Discard, o!()));
        looper.loop_forever();
        assert_eq!(0, *logic.handle_error.borrow());
        assert_eq!(0, *logic.in_progress.borrow());
        assert_eq!(0, *logic.not_candidate.borrow());
        assert_eq!(21, *logic.post_check.borrow());
        assert_eq!(21, *logic.pre_check.borrow());
        assert_eq!(5, *logic.primary.borrow());
        assert_eq!(16, *logic.secondary.borrow());
        assert_eq!(0, *logic.terminated.borrow());
    }

    #[test]
    fn shutdown_signal() {
        let mock_coordinator = mock_coordinator();
        let coordinator = mock_coordinator.mock();
        let (shutdown, receiver) = LoopingElectionOpts::shutdown_channel();
        let handle = ::std::thread::spawn(move || {
            let election = coordinator.election("test");
            let logic = {
                let logic = TestLogic::new();
                *logic.max_loops.borrow_mut() = 1;
                logic
            };
            let opts = LoopingElectionOpts::new(election, logic.clone())
                .election_term(5)
                .loop_delay(Duration::from_secs(2))
                .shutdown_receiver(receiver);
            let mut looper = LoopingElection::new(opts, Logger::root(Discard, o!()));
            looper.loop_forever();
            assert_eq!(0, *logic.handle_error.borrow());
            assert_eq!(0, *logic.in_progress.borrow());
            assert_eq!(1, *logic.not_candidate.borrow());
            assert_eq!(1, *logic.post_check.borrow());
            assert_eq!(1, *logic.pre_check.borrow());
            assert_eq!(0, *logic.primary.borrow());
            assert_eq!(0, *logic.secondary.borrow());
            assert_eq!(0, *logic.terminated.borrow());
        });
        shutdown.request();
        handle.join().unwrap();
    }
}
