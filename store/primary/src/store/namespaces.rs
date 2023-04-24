use opentracingrust::SpanContext;

use replicante_models_core::scope::Namespace;

use super::super::backend::NamespacesImpl;
use super::super::Cursor;
use super::super::Result;

/// Operate on all nodes in the cluster identified by cluster_id.
pub struct Namespaces {
    namespaces: NamespacesImpl,
}

impl Namespaces {
    pub(crate) fn new(namespaces: NamespacesImpl) -> Namespaces {
        Namespaces { namespaces }
    }

    /// Iterate over the namespace on the RepliCore instance.
    pub fn iter<S>(&self, span: S) -> Result<Cursor<Namespace>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.namespaces.iter(span.into())
    }
}
