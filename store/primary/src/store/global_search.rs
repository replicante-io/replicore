use opentracingrust::SpanContext;

use replicante_models_core::cluster::discovery::DiscoverySettings;

use crate::backend::GlobalSearchImpl;
use crate::Cursor;
use crate::Result;

pub struct GlobalSearch {
    search: GlobalSearchImpl,
}

impl GlobalSearch {
    pub(crate) fn new(search: GlobalSearchImpl) -> GlobalSearch {
        GlobalSearch { search }
    }

    /// Iterate over `DiscoverySettings` waiting to be scheduled.
    pub fn discoveries_to_run<S>(&self, span: S) -> Result<Cursor<DiscoverySettings>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.search.discoveries_to_run(span.into())
    }
}
