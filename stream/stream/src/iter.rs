use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use humthreads::ThreadScope;
use rand::Rng;
use sentry;
use sentry::protocol::Breadcrumb;
use sentry::protocol::Map;
use serde::de::DeserializeOwned;

use crate::metrics::BACKOFF_DURATION;
use crate::metrics::BACKOFF_REPEAT;
use crate::metrics::BACKOFF_REQUIRED;
use crate::metrics::BACKOFF_TOTAL;
use crate::metrics::DELIVERED_ERROR;
use crate::metrics::DELIVERED_RETRY;
use crate::metrics::DELIVERED_TOTAL;
use crate::Message;
use crate::Result;

/// Iterator over `Message`s emitted to the stream.
///
/// # Errors and retries
/// Streams have the property of being ordered with respect to a key.
/// Streaming platforms ensure that as long as consumers of the stream respect
/// the ordering given by the platform and don't start jumping around.
/// That means that consumers MUST process a message before they move on to the
/// next or they will miss messages or mess up the ordering guarantees.
///
/// To help stream followers process messages in order, even in the presence
/// of transient failures, `Iter` instances will return the same message
/// multiple times if the code consuming the messages fails to acknowledge them:
///
/// ```ignore
/// // scope is a humthreads::ThreadScope passed to the caller.
/// let iter = stream.follow("follower.group", &scope)?;
/// for message in iter {
///     let message = message.expect("unable to follow stream");
///     if scope.should_shutdown() {
///         return;
///     }
///     if let Err(e) = some_operation_that_may_fail(message) {
///         log!("Error processing message@ {:?}", e);
///         // By moving to the next message without acknowledging the current one the
///         // iterator will return the same message, giving us a canche to try again.
///         continue;
///     }
///     message.async_ack().expect("unable to ack message");
/// }
/// ```
///
/// The `Iter` instance is responsible to ensure messages are re-delivered as well as
/// implementing a backoff strategy instead of aggressively failing continuously.
/// In case of transient failures (MongoDB failover, network partition, etc ...) this
/// should be enough to eventually succeed a processing tasks with minimal effort
/// from the message consuming code.
///
/// # Backoff strategy
/// The backoff strategy is similar to
/// [exponential backoff](https://en.wikipedia.org/wiki/Exponential_backoff)
/// but not quite the same.
///
/// Like with exponential backoff, the delay is random and the retry attempt is
/// used as the exponent for a fixed base.
///
/// Unkile with exponential backoff, the delay is selected at random within a
/// range `[base ^ (attemt - 1), base ^ attempt)` instead of `[0, max)`.
/// The backoff strategy also sets caps to prevent the "top range" from growing too much:
/// blocking an entire stream for large amounts of time is not a reasonable approach here.
///
/// # Panics
/// If the failure processing a message is persistent, such as a bug in the code or a
/// prolonged outage in one of the dependencies, retrying forever won't solve the problem.
/// Because skipping messages and hoping for the best is NOT what streams are about,
/// we have async tasks for that, the only thing that `Iter` instances can do is panic.
///
/// In this cases, the entire Replicante Core process will exit so end-users and operators
/// can become aware of the issue in a timely manner.
/// The solution to the problem varies based on the exact nature of the problem:
///
///   * If one of the dependencies has failed, Replicante Core will be able to proceed
///     only once that is fixed.
///   * If the issue is a software bug with a known fix, update to it.
///   * If there is no other way to solve the issue, manually acknowledge the failing
///     message with the streaming platform tools to force Replicante Core to skip it.
///     This option should be used only by advanced users or when directed to do so.
///     Consequences often apply and should be understood before proceeding.
pub struct Iter<'a, T>
where
    T: DeserializeOwned + 'static,
{
    backoff: Backoff,
    follow_id: String,
    in_progress: Option<Message<T>>,
    in_progress_acked: Rc<RefCell<bool>>,
    inner: Box<dyn Iterator<Item = Result<Message<T>>> + 'a>,
    stream_id: &'static str,
    thread: &'a ThreadScope,
}

impl<'a, T> Iter<'a, T>
where
    T: DeserializeOwned + 'static,
{
    pub(crate) fn with_iter(
        stream_id: &'static str,
        follow_id: String,
        backoff: Backoff,
        thread: &'a ThreadScope,
        iter: Box<dyn Iterator<Item = Result<Message<T>>> + 'a>,
    ) -> Iter<'a, T> {
        Iter {
            backoff,
            follow_id,
            in_progress: None,
            in_progress_acked: Rc::new(RefCell::new(true)),
            inner: iter,
            stream_id,
            thread,
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: DeserializeOwned + 'static,
{
    type Item = Result<Message<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        let message_done: bool = *self.in_progress_acked.borrow();
        let no_message: bool = self.in_progress.is_none();
        if message_done || no_message {
            self.in_progress = match self.inner.next() {
                Some(Ok(message)) => Some(message),
                None => None,
                error => {
                    DELIVERED_ERROR
                        .with_label_values(&[self.stream_id, &self.follow_id])
                        .inc();
                    self.backoff.wait(
                        self.thread,
                        self.stream_id,
                        &self.follow_id,
                        "<fetch error>",
                    );
                    return error;
                }
            };
            self.backoff.reset(self.stream_id, &self.follow_id);
            self.in_progress_acked = self
                .in_progress
                .as_ref()
                .map(|in_progress| Rc::clone(&in_progress.notify_message_acked))
                .unwrap_or_else(|| Rc::new(RefCell::new(true)));
        } else {
            let message_id = self
                .in_progress
                .as_ref()
                .expect("need message for re-delvery")
                .id();
            self.backoff
                .wait(self.thread, self.stream_id, &self.follow_id, &message_id);
            DELIVERED_RETRY
                .with_label_values(&[self.stream_id, &self.follow_id])
                .inc();
        }
        self.in_progress.clone().map(|message| {
            DELIVERED_TOTAL
                .with_label_values(&[self.stream_id, &self.follow_id])
                .inc();
            Ok(message)
        })
    }
}

// Should I use https://crates.io/crates/exponential-backoff ?
pub struct Backoff {
    attempt: u32,
    attempts_limit: u32,
    base: u32,
    cap_max: Duration,
    cap_min: Duration,
    increment: Duration,
}

impl Backoff {
    /// Default backoff strategy.
    pub fn new() -> Backoff {
        Backoff {
            attempt: 0,
            attempts_limit: 8,
            base: 2,
            cap_max: Duration::from_secs(300),
            cap_min: Duration::from_secs(240),
            increment: Duration::from_secs(5),
        }
    }

    /// Backoff strategy that never sleeps for tests speed.
    #[cfg(any(feature = "with_test_support", test))]
    pub fn fast() -> Backoff {
        Backoff {
            attempt: 0,
            attempts_limit: 8,
            base: 2,
            cap_max: Duration::from_secs(10),
            cap_min: Duration::from_secs(1),
            increment: Duration::from_secs(0),
        }
    }

    /// Reset the backoff attempts count after a successful operation.
    pub fn reset(&mut self, stream_id: &str, group: &str) {
        BACKOFF_REQUIRED
            .with_label_values(&[stream_id, group])
            .observe(f64::from(self.attempt));
        self.attempt = 0;
    }

    /// Wait for an appropriate backoff time.
    ///
    /// Additionally, update the internal state to backoff more if called again.
    pub fn wait(&mut self, thread: &ThreadScope, stream_id: &str, group: &str, message_id: &str) {
        self.check_limit(stream_id, group, message_id);
        BACKOFF_TOTAL.with_label_values(&[stream_id, group]).inc();
        if self.attempt > 0 {
            BACKOFF_REPEAT.with_label_values(&[stream_id, group]).inc();
        }
        self.attempt += 1;
        let start = Instant::now();
        let delay = self.delay();
        let _activity = thread.scoped_activity(format!(
            "backing off stream follower for {} seconds",
            delay.as_secs()
        ));
        let _timer = BACKOFF_DURATION
            .with_label_values(&[stream_id, group])
            .start_timer();
        while start.elapsed() < delay && !thread.should_shutdown() {
            thread::sleep(Duration::from_millis(500));
        }
    }

    fn check_limit(&self, stream_id: &str, group: &str, message_id: &str) {
        if self.attempt > self.attempts_limit {
            sentry::with_scope(
                |_| (),
                || {
                    sentry::add_breadcrumb(Breadcrumb {
                        category: Some("stream.follower".into()),
                        message: Some("Stream unable to handle message".into()),
                        data: {
                            let mut map = Map::new();
                            map.insert("stream_id".into(), stream_id.into());
                            map.insert("following_group".into(), group.into());
                            map.insert("message_id".into(), message_id.into());
                            map
                        },
                        ..Default::default()
                    });
                    panic!(
                        "Stream unable to handle message; stream={}, group={}, message={}",
                        stream_id, group, message_id,
                    );
                },
            );
        }
    }

    fn delay(&self) -> Duration {
        let min = self.min_delay();
        let max = self.max_delay();
        if min == max {
            return min;
        }
        rand::thread_rng().gen_range(min, max)
    }

    fn max_delay(&self) -> Duration {
        let max = self.increment * self.base.pow(self.attempt);
        max.min(self.cap_max)
    }

    fn min_delay(&self) -> Duration {
        let min = self.increment * self.base.pow(self.attempt - 1);
        min.min(self.cap_min)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use humthreads::test_support::MockThreadScope;

    use super::Backoff;

    #[test]
    fn limit_is_incremented() {
        let scope = MockThreadScope::new();
        let mut backoff = Backoff::new();
        backoff.attempts_limit = 10;
        backoff.increment = Duration::from_secs(0);
        backoff.wait(&scope.scope(), "stream", "test", "par1/off5");
        backoff.wait(&scope.scope(), "stream", "test", "par1/off5");
        backoff.wait(&scope.scope(), "stream", "test", "par1/off5");
        assert_eq!(backoff.attempt, 3);
    }

    #[test]
    fn max_delay() {
        let mut backoff = Backoff::new();
        backoff.base = 2;
        backoff.cap_max = Duration::from_secs(600);
        backoff.increment = Duration::from_secs(1);
        backoff.attempt = 3;
        assert_eq!(backoff.max_delay(), Duration::from_secs(8));
        backoff.attempt = 6;
        assert_eq!(backoff.max_delay(), Duration::from_secs(64));
        backoff.attempt = 10;
        assert_eq!(backoff.max_delay(), Duration::from_secs(600));
    }

    #[test]
    fn min_delay() {
        let mut backoff = Backoff::new();
        backoff.base = 2;
        backoff.cap_min = Duration::from_secs(100);
        backoff.increment = Duration::from_secs(1);
        backoff.attempt = 3;
        assert_eq!(backoff.min_delay(), Duration::from_secs(4));
        backoff.attempt = 6;
        assert_eq!(backoff.min_delay(), Duration::from_secs(32));
        backoff.attempt = 10;
        assert_eq!(backoff.min_delay(), Duration::from_secs(100));
    }

    #[test]
    #[should_panic(
        expected = "Stream unable to handle message; stream=stream, group=test, message=par1/off5"
    )]
    fn panic_after_limit() {
        let mut backoff = Backoff::new();
        backoff.attempt = 5;
        backoff.attempts_limit = 1;
        backoff.check_limit("stream", "test", "par1/off5");
    }

    #[test]
    fn shutdown_flag_is_checked() {
        let mut backoff = Backoff::new();
        backoff.attempt = 3;
        backoff.base = 2;
        backoff.cap_max = Duration::from_secs(10);
        backoff.cap_min = Duration::from_secs(8);
        backoff.increment = Duration::from_secs(2);
        let scope = MockThreadScope::new();
        scope.set_shutdown(true);
        // Time the wait to ensure it is less then the periodic sleep + a margin of error.
        let start = std::time::Instant::now();
        backoff.wait(&scope.scope(), "stream", "test", "par1/off5");
        assert!(
            start.elapsed() < Duration::from_secs(1),
            "backoff did not return in time"
        );
    }
}
