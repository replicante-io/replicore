use opentracingrust::SpanContext;
use replisdk::core::models::platform::Platform as PlatformModel;

use super::super::backend::PlatformsImpl;
use super::super::Cursor;
use super::super::Result;

/// Operate on all `Platform`s in the given namespace.
pub struct Platforms {
    platforms: PlatformsImpl,
    attrs: PlatformsAttributes,
}

impl Platforms {
    pub(crate) fn new(platforms: PlatformsImpl, attrs: PlatformsAttributes) -> Platforms {
        Platforms { platforms, attrs }
    }

    /// Iterate over platforms in a cluster.
    pub fn iter<S>(&self, span: S) -> Result<Cursor<PlatformModel>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.platforms.iter(&self.attrs, span.into())
    }
}

/// Attributes attached to all platforms operations.
pub struct PlatformsAttributes {
    pub ns_id: String,
}
