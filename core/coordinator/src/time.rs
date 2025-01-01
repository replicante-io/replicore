//! Exclusive logic that repeats on a timer.
use std::time::Duration;

use anyhow::Result;
use replisdk::runtime::shutdown::ShutdownHandle;

use replicore_context::Context;

use crate::ICoordinated;

/// Interface for coordinated logic to execute repeatedly on a timer.
#[async_trait::async_trait]
pub trait ICoordinatedTimer: Sync {
    /// Execute an occurrence of the recursive logic.
    async fn tick(&self, context: &Context);
}

/// Coordinated object that repeatedly invokes exclusive logic on a timer on the primary process.
pub struct Timer<L>
where
    L: ICoordinatedTimer,
{
    /// Handle to receive graceful shutdown notification.
    exit_handle: ShutdownHandle,

    /// Delay to wait between executions of the timed logic.
    interval: Duration,

    /// Implementation of the timed logic to execute at every loop.
    timed_logic: L,
}

impl<L> Timer<L>
where
    L: ICoordinatedTimer,
{
    /// Initialise a new coordinated timer.
    pub fn new(logic: L, interval: Duration, exit: ShutdownHandle) -> Timer<L> {
        Timer {
            exit_handle: exit,
            interval,
            timed_logic: logic,
        }
    }
}

#[async_trait::async_trait]
impl<L> ICoordinated for Timer<L>
where
    L: ICoordinatedTimer,
{
    async fn primary(&self, context: &Context) -> Result<()> {
        let exit_handle = self.exit_handle.clone().wait();
        tokio::pin!(exit_handle);

        loop {
            self.timed_logic.tick(context).await;
            tokio::select! {
                _ = tokio::time::sleep(self.interval) => (),
                _ = &mut exit_handle => return Ok(()),
            }
        }
    }

    async fn secondary(&self, _: &Context) -> Result<()> {
        self.exit_handle.clone().wait().await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::time::Duration;

    use replisdk::runtime::shutdown::ShutdownHandle;

    use replicore_context::Context;

    use crate::ICoordinated;
    use crate::ICoordinatedTimer;
    use crate::Timer;

    /// Mock timer that just increments a counter.
    #[derive(Clone)]
    struct Count {
        /// Keep track of timer invocations.
        state: Arc<Mutex<usize>>,
    }

    impl Count {
        fn peek(&self) -> usize {
            *self.state.lock().expect("Count lock poisoned")
        }
    }

    #[async_trait::async_trait]
    impl ICoordinatedTimer for Count {
        async fn tick(&self, _: &Context) {
            let mut state = self.state.lock().expect("Count lock poisoned");
            *state += 1;
        }
    }

    impl Default for Count {
        fn default() -> Self {
            let state = Arc::new(Mutex::new(0));
            Count { state }
        }
    }

    #[tokio::test]
    async fn primary_ticks_on_loop() {
        let context = Context::fixture();
        let (exit, signal) = ShutdownHandle::fixture();
        let interval = Duration::from_millis(2);
        let logic = Count::default();
        let timer = Timer::new(logic.clone(), interval, exit);

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(7)).await;
            signal.send(true).unwrap();
        });
        timer.primary(&context).await.unwrap();

        let count = logic.peek();
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn secondary_exits_on_signal() {
        let context = Context::fixture();
        let (exit, signal) = ShutdownHandle::fixture();
        let interval = Duration::from_millis(2);
        let logic = Count::default();
        let timer = Timer::new(logic.clone(), interval, exit);

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(7)).await;
            signal.send(true).unwrap();
        });
        timer.secondary(&context).await.unwrap();

        let count = logic.peek();
        assert_eq!(count, 0);
    }
}
