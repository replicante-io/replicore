use std::collections::HashMap;
use std::sync::Arc;

use failure::ResultExt;
use opentracingrust::InjectFormat;
use opentracingrust::Result as OTResult;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;
use serde::Serialize;

use replicante_service_healthcheck::HealthChecks;

use super::config::Backend as BackendConfig;
use super::metrics::TASK_REQUEST_ERRORS;
use super::metrics::TASK_REQUEST_TOTAL;
use super::Config;
use super::ErrorKind;
use super::Result;
use super::TaskId;
use super::TaskQueue;

mod backend;

use self::backend::kafka::Kafka;
use self::backend::Backend;

/// Request a task to be queued for processing
pub struct TaskRequest<Q: TaskQueue> {
    headers: HashMap<String, String>,
    id: TaskId,
    queue: Q,
}

impl<Q: TaskQueue> TaskRequest<Q> {
    /// Attach or update an header to the task.
    pub fn header<S1, S2>(&mut self, header: S1, value: S2)
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        self.headers.insert(header.into(), value.into());
    }

    /// Attach or update a set of headers to the task.
    pub fn headers(&mut self, headers: HashMap<String, String>) {
        for (key, value) in headers {
            self.headers.insert(key, value);
        }
    }

    /// Access the ID of this task.
    pub fn id(&self) -> &TaskId {
        &self.id
    }

    /// Create a new task for the given queue and carring the given message.
    pub fn new(queue: Q) -> TaskRequest<Q> {
        TaskRequest {
            headers: HashMap::new(),
            id: TaskId::new(),
            queue,
        }
    }

    /// Access information about the task's queue.
    pub fn queue(&self) -> &Q {
        &self.queue
    }

    /// Inject a span context into the task request.
    ///
    /// Tasks handlers can then extract the span context to provide a full
    /// trace of the larger task across processes/systems.
    #[allow(clippy::result_large_err)]
    pub fn trace(&mut self, context: &SpanContext, tracer: &Tracer) -> OTResult<()> {
        let mut headers = HashMap::new();
        let format = InjectFormat::HttpHeaders(Box::new(&mut headers));
        tracer.inject(context, format)?;
        self.headers(headers);
        Ok(())
    }
}

/// Manages task requests to the queue system.
#[derive(Clone)]
pub struct Tasks<Q: TaskQueue>(Arc<dyn Backend<Q>>);

impl<Q: TaskQueue> Tasks<Q> {
    /// Create a new `Tasks` interface to enqueue new tasks.
    pub fn new(config: Config, healthchecks: &mut HealthChecks) -> Result<Tasks<Q>> {
        let backend = match config.backend {
            BackendConfig::Kafka(backend) => Arc::new(Kafka::new(backend, healthchecks)?),
        };
        Ok(Tasks(backend))
    }

    /// Request a new task to be performed.
    ///
    /// Tasks are performed asynchronously and, likely, in separate processes.
    /// There is no guarantee about times within which tasks are completed.
    pub fn request<M: Serialize>(&self, task: TaskRequest<Q>, message: M) -> Result<()> {
        let message =
            ::serde_json::to_vec(&message).with_context(|_| ErrorKind::PayloadSerialize)?;
        let queue = task.queue.name();
        TASK_REQUEST_TOTAL.with_label_values(&[&queue]).inc();
        self.0.request(task, &message).map_err(|error| {
            TASK_REQUEST_ERRORS.with_label_values(&[&queue]).inc();
            error
        })
    }
}

#[cfg(any(debug_assertions, feature = "with_test_support"))]
pub type MockedRequests<Q> = Arc<::std::sync::Mutex<Vec<(TaskRequest<Q>, ::serde_json::Value)>>>;

/// Mock tools to test `Tasks` users.
#[cfg(any(debug_assertions, feature = "with_test_support"))]
#[derive(Default)]
pub struct MockTasks<Q: TaskQueue> {
    pub requests: MockedRequests<Q>,
}

#[cfg(any(debug_assertions, feature = "with_test_support"))]
impl<Q: TaskQueue> MockTasks<Q> {
    /// Create a mock tasks instance to be used for tests.
    pub fn new() -> MockTasks<Q> {
        MockTasks {
            requests: Arc::new(::std::sync::Mutex::new(Vec::new())),
        }
    }

    /// Return the non-mock interface to interact with this mock.
    pub fn mock(&self) -> Tasks<Q> {
        Tasks(Arc::new(self::backend::mock::Mock {
            requests: self.requests.clone(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::time::Duration;

    use super::MockTasks;
    use super::TaskQueue;
    use super::TaskRequest;

    #[derive(Clone, Debug, Eq, Hash, PartialEq)]
    enum TestQueues {
        Test,
    }

    impl FromStr for TestQueues {
        type Err = ::failure::Error;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "test" => Ok(TestQueues::Test),
                s => Err(::failure::err_msg(format!("unknown queue '{}'", s))),
            }
        }
    }

    impl TaskQueue for TestQueues {
        fn max_retry_count(&self) -> u8 {
            12
        }
        fn name(&self) -> String {
            match self {
                TestQueues::Test => "test".into(),
            }
        }
        fn retry_delay(&self) -> Duration {
            Duration::from_secs(5 * 60)
        }
    }

    #[test]
    fn enqueue_request() {
        let task = TaskRequest::new(TestQueues::Test);
        let message: String = "Some text".into();
        let mock: MockTasks<TestQueues> = MockTasks::new();
        mock.mock()
            .request(task, message)
            .expect("failed to request task");
        let found = &mock.requests.lock().expect("failed to lock")[0];
        assert_eq!(found.0.queue(), &TestQueues::Test);
        assert_eq!("Some text", found.1);
    }

    #[test]
    fn request_unit() {
        let task = TaskRequest::new(TestQueues::Test);
        let mock: MockTasks<TestQueues> = MockTasks::new();
        mock.mock()
            .request(task, ())
            .expect("failed to request task");
        let found = &mock.requests.lock().expect("failed to lock")[0];
        assert_eq!(found.0.queue(), &TestQueues::Test);
    }
}
