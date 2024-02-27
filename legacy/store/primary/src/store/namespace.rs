use opentracingrust::SpanContext;

use replicante_models_core::scope::Namespace as NamespaceModel;

use super::super::backend::NamespaceImpl;
use super::super::Result;

/// Operate on the namespace with the provided ID.
pub struct Namespace {
    namespace: NamespaceImpl,
    attrs: NamespaceAttributes,
}

impl Namespace {
    pub(crate) fn new(namespace: NamespaceImpl, attrs: NamespaceAttributes) -> Namespace {
        Namespace { namespace, attrs }
    }

    /// Query the `Namespace` record, if any is stored.
    pub fn get<S>(&self, span: S) -> Result<Option<NamespaceModel>>
    where
        S: Into<Option<SpanContext>>,
    {
        let namespace = self.namespace.get(&self.attrs, span.into());

        // TODO(namespace-rollout): Stop injecting default namespace here.
        if let Ok(None) = namespace {
            if self.attrs.ns_id == "default" {
                let default = NamespaceModel::HARDCODED_FOR_ROLLOUT();
                return Ok(Some(default));
            }
        }

        namespace
    }
}

/// Attributes attached to all namespace operations.
pub struct NamespaceAttributes {
    pub ns_id: String,
}
