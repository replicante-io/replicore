use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread::Builder;
use std::thread::JoinHandle;
use std::time::Duration;

use slog::Logger;

use super::super::Config;
use super::super::config::Backend as BackendConfig;

use super::Result;
use super::Task;
use super::TaskError;
use super::TaskQueue;

use super::backend::Backend;
use super::backend::kafka::Kafka;


#[cfg(debug_assertions)]
pub mod mock;


const TIMEOUT_MS_POLL: u64 = 500;
const TIMEOUT_MS_ERROR: u64 = 1000;


/// Interface for code that can process a task.
pub trait TaskHandler<Q: TaskQueue> : Send + Sync + 'static {
    /// Process the given task.
    ///
    /// The handler MUST call one of the task acknowledgement methods (`fail`, `skip`, `success`)
    /// or the worker thread will panic to ensure that no task is skipped.
    fn handle(&self, task: Task<Q>);
}

impl<F, Q> TaskHandler<Q> for F
    where F: Fn(Task<Q>) -> () + Send + Sync + 'static,
          Q: TaskQueue,
{
    fn handle(&self, task: Task<Q>) {
        self(task)
    }
}


/// Worker logic run by each thread.
struct Worker<Q: TaskQueue> {
    backend: Arc<Backend<Q>>,
    handlers: Arc<HashMap<Q, Box<TaskHandler<Q>>>>,
    logger: Logger,
}

impl<Q: TaskQueue> Worker<Q> {
    fn new(
        logger: Logger, backend: Arc<Backend<Q>>, handlers: Arc<HashMap<Q, Box<TaskHandler<Q>>>>
    ) -> Worker<Q> {
        Worker {
            backend,
            handlers,
            logger,
        }
    }

    /// Perform a single "worker cycle".
    fn run_once(&self) {
        let task = match self.backend.poll(Duration::from_millis(TIMEOUT_MS_POLL)) {
            Err(error) => {
                error!(self.logger, "Failed to poll for tasks, sleeping before retring"; "error" => ?error);
                ::std::thread::sleep(Duration::from_millis(TIMEOUT_MS_ERROR));
                return
            },
            Ok(None) => return,
            Ok(Some(task)) => task,
        };
        trace!(self.logger, "Received task"; "queue" => task.queue.name());
        match self.handlers.get(&task.queue) {
            None => error!(self.logger, "No task handler found"; "queue" => task.queue.name()),
            Some(handler) => handler.handle(task),
        };
    }
}


/// Builder for a worker threads pool receiving and processing tasks.
pub struct WorkerSet<Q: TaskQueue> {
    backend: Arc<Backend<Q>>,
    config: Config,
    handlers: HashMap<Q, Box<TaskHandler<Q>>>,
    logger: Logger,
}

impl<Q: TaskQueue> WorkerSet<Q> {
    pub fn new(logger: Logger, config: Config) -> Result<WorkerSet<Q>> {
        let backend = match config.backend.clone() {
            BackendConfig::Kafka(backend) => Arc::new(Kafka::new(backend, logger.clone())?),
        };
        Ok(WorkerSet {
            backend,
            config,
            handlers: HashMap::new(),
            logger,
        })
    }

    /// Start the threads pool and wait for tasks to process.
    pub fn run(self) -> Result<WorkerSetPool> {
        let handlers = Arc::new(self.handlers);
        let running = Arc::new(AtomicBool::new(true));
        let mut threads = Vec::new();

        for idx in 0..self.config.threads_count {
            let logger = self.logger.clone();
            let name = format!("r:c:tasks:worker:{}", idx);
            let still_running = Arc::clone(&running);
            let thread_backend = Arc::clone(&self.backend);
            let thread_handlers = Arc::clone(&handlers);

            let thread = Builder::new().name(name).spawn(move || {
                let worker: Worker<Q> = Worker::new(logger, thread_backend, thread_handlers);
                while still_running.load(Ordering::SeqCst) {
                    worker.run_once();
                }
            });
            threads.push(thread);
        }

        // If any of the threads failed to spawn we need to terminate the pool and clean up.
        if threads.iter().any(|t| t.is_err()) {
            running.store(false, Ordering::SeqCst);
            for thread in threads.into_iter() {
                if let Ok(handle) = thread {
                    // TODO: propagate error when we have a better story?
                    if let Err(error) = handle.join() {
                        error!(self.logger, "WorkerSet pool thread paniced"; "error" => ?error);
                    }
                }
            }
            return Err(TaskError::Msg("could not sapwn all worker threads".into()).into());
        }

        // All threads where spawned successfully so we can unwrap the reustl.
        let mut handles = Vec::new();
        for t in threads {
            let t = t.expect("spawn errors should have been managed above!");
            handles.push(t);
        }

        Ok(WorkerSetPool {
            logger: self.logger.clone(),
            running,
            threads: handles,
        })
    }
    
    /// Register a new worker routine for a queue.
    ///
    /// Each queue can only have one handling routine associated with it.
    /// Only queues that have a handler attached to them will be subscirbed to.
    /// Nothing prevents the same handler from being used to process multiple queues.
    ///
    /// Handlers are shared across the thread pool and may be executed multiple times in parallel.
    ///
    /// The number of `worker` calls has no impact on how many threads are created
    /// (see the configuration for that).
    pub fn worker<H: TaskHandler<Q>>(mut self, queue: Q, handler: H) -> Result<Self> {
        Arc::get_mut(&mut self.backend)
            .expect("there should only be one reference to the backend at this point")
            .subscribe(&queue)?;
        self.handlers.insert(queue, Box::new(handler));
        Ok(self)
    }
}


/// Set of worker threads processing tasks.
pub struct WorkerSetPool {
    logger: Logger,
    running: Arc<AtomicBool>,
    threads: Vec<JoinHandle<()>>,
}

impl WorkerSetPool {
    /// Stop the background thread pool and wait for threads to terminate.
    pub fn stop(&mut self) -> Result<()> {
        self.running.store(false, Ordering::SeqCst);
        while let Some(handle) = self.threads.pop() {
            // TODO: propagate error when we have a better story?
            if let Err(error) = handle.join() {
                error!(self.logger, "WorkerSet pool thread paniced"; "error" => ?error);
            }
        }
        Ok(())
    }

    // TODO: implement utilities that make this possible.
    //pub fn wait(&self, _timeout: Duration) -> Result<ThreadStatus> {
    //    Err(TODO)
    //}
}

impl Drop for WorkerSetPool {
    fn drop(&mut self) {
        if let Err(error) = self.stop() {
            error!(self.logger, "Failed to stop WorkerSet pool on drop"; "error" => ?error);
        }
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::str::FromStr;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::time::Duration;

    use slog::Discard;
    use slog::Logger;

    use super::super::mock::MockWorkerSet;
    use super::super::mock::TaskTemplate;
    use super::TIMEOUT_MS_POLL;
    use super::Task;
    use super::TaskQueue;

    #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    enum TestQueues {
        Test1,
        Test2,
    }

    impl FromStr for TestQueues {
        type Err = ::failure::Error;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "test1" => Ok(TestQueues::Test1),
                "test2" => Ok(TestQueues::Test2),
                s => Err(::failure::err_msg(format!("unknown queue '{}'", s))),
            }
        }
    }

    impl TaskQueue for TestQueues {
        fn name(&self) -> String {
            match self {
                TestQueues::Test1 => "test1".into(),
                TestQueues::Test2 => "test2".into(),
            }
        }
    }

    #[test]
    fn dispath_task() {
        let logger = Logger::root(Discard, o!());
        let task = TaskTemplate::new(TestQueues::Test1, (), HashMap::new(), 0);
        let mock_set = MockWorkerSet::new();
        (*mock_set.tasks.lock().unwrap()).push_back(task);
        let processed = Arc::new(Mutex::new(Vec::new()));
        let processed_thread = Arc::clone(&processed);
        let _workers = mock_set.mock(logger)
            .worker(TestQueues::Test1, move |task: Task<TestQueues>| {
                let queue = task.queue.name();
                processed_thread.lock().unwrap().push(queue);
            }).unwrap()
            .run().unwrap();
        ::std::thread::sleep(Duration::from_millis(TIMEOUT_MS_POLL + 100));
        assert_eq!(*processed.lock().unwrap(), vec![String::from("test1")]);
    }

    #[test]
    fn map_queue_to_handler() {
        let logger = Logger::root(Discard, o!());
        let mock_set = MockWorkerSet::new();
        let workers = mock_set.mock(logger)
            .worker(TestQueues::Test1, |_|()).unwrap()
            .worker(TestQueues::Test2, |_| ()).unwrap();
        assert_eq!(workers.handlers.len(), 2);
        let mut keys: Vec<TestQueues> = workers.handlers.keys().map(|k| k.clone()).collect();
        keys.sort();
        assert_eq!(keys, vec![TestQueues::Test1, TestQueues::Test2]);
    }

    #[test]
    fn stop_pool() {
        let logger = Logger::root(Discard, o!());
        let mock_set = MockWorkerSet::new();
        let mut workers = mock_set.mock(logger)
            .worker(TestQueues::Test1, |_| ()).unwrap()
            .run().unwrap();
        workers.stop().unwrap();
    }
}
