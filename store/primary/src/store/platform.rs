use opentracingrust::SpanContext;
use replisdk::core::models::platform::Platform as PlatformModel;

use super::super::backend::PlatformImpl;
use super::super::Result;

/// Operate on the [`Platform`](PlatformModel) identified by the provided namespace and platform.
pub struct Platform {
    platform: PlatformImpl,
    attrs: PlatformAttributes,
}

impl Platform {
    pub(crate) fn new(platform: PlatformImpl, attrs: PlatformAttributes) -> Platform {
        Platform { platform, attrs }
    }

    /// Query the [`Platform`](PlatformModel) record, if any is stored.
    pub fn get<S>(&self, span: S) -> Result<Option<PlatformModel>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.platform.get(&self.attrs, span.into())
    }
}

/// Attributes attached to all platform operations.
pub struct PlatformAttributes {
    pub ns_id: String,
    pub platform_id: String,
}
