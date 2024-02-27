use opentracingrust::SpanContext;

use crate::backend::DiscoverySettingsImpl;
use crate::Cursor;
use crate::Result;

/// Operate on discovery settings models.
pub struct DiscoverySettings {
    attrs: DiscoverySettingsAttributes,
    settings: DiscoverySettingsImpl,
}

impl DiscoverySettings {
    pub(crate) fn new(
        settings: DiscoverySettingsImpl,
        attrs: DiscoverySettingsAttributes,
    ) -> DiscoverySettings {
        DiscoverySettings { attrs, settings }
    }

    /// Delete the named DiscoverySettings object.
    pub fn delete<S>(&self, name: &str, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.settings.delete(&self.attrs, name, span.into())
    }

    /// Iterate over the names of DiscoverySettings objects in the namespace.
    ///
    /// Names are returned in ascending alphabetical order.
    pub fn iter_names<S>(&self, span: S) -> Result<Cursor<String>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.settings.iter_names(&self.attrs, span.into())
    }
}

/// Attributes attached to all discovery settings operations.
pub struct DiscoverySettingsAttributes {
    pub namespace: String,
}
