use replicante_models_core::api::validate::ErrorsCollection;

/// Error indicating a context does not exist.
#[derive(thiserror::Error, Debug)]
#[error("A context named '{context}' was not found")]
pub struct ContextNotFound {
    context: String,
}

impl ContextNotFound {
    /// Create a context not found error for the given name.
    pub fn for_name<S>(name: S) -> ContextNotFound
    where
        S: Into<String>,
    {
        let context = name.into();
        ContextNotFound { context }
    }

    /// The name of the context we failed to find.
    pub fn name(&self) -> &str {
        &self.context
    }
}

/// Apply attempted on an invalid object.
#[derive(thiserror::Error, Debug)]
#[error("Apply attempted on an invalid object")]
pub struct InvalidApply {
    errors: replicante_models_core::api::validate::ErrorsCollection,
}

impl InvalidApply {
    pub fn new(errors: ErrorsCollection) -> InvalidApply {
        InvalidApply { errors }
    }
}

impl std::ops::Deref for InvalidApply {
    type Target = [replicante_models_core::api::validate::Error];
    fn deref(&self) -> &Self::Target {
        self.errors.deref()
    }
}
