//! Callback invoked when platform discovery task need to be executed.
use anyhow::Result;

use replicore_context::Context;
use replicore_injector::Injector;
use replicore_tasks::execute::ReceivedTask;
use replicore_tasks::execute::TaskCallback;

use crate::clients::Clients;
use crate::DiscoverPlatform;

/// Callback to execute platform discovery tasks.
pub struct Callback {
    pub(crate) clients: Clients,
    pub(crate) injector: Injector,
}

impl Default for Callback {
    fn default() -> Self {
        let mut clients = Clients::default();
        clients
            .with_url_factory("http", crate::clients::HttpClientFactory)
            .with_url_factory("https", crate::clients::HttpClientFactory);

        let injector = Injector::global();
        Self { clients, injector }
    }
}

#[async_trait::async_trait]
impl TaskCallback for Callback {
    async fn execute(&self, context: &Context, task: &ReceivedTask) -> Result<()> {
        let request: DiscoverPlatform = task.decode()?;
        slog::debug!(
            context.logger, "Reached platform discovery task callback";
            "request" => ?request,
        );
        // TODO(locking): exit early if platform already under discovery.

        crate::discover::discover(context, self, request).await
    }
}
