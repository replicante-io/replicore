use anyhow::Result;
use tera::Tera;

/// Errors related to template loading and rendering.
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("path to template is not valid unicode")]
    NonUnicodePath,
}

/// Load a [`Tera`] template.
#[derive(Default)]
pub struct TemplateLoader {}

#[async_trait::async_trait]
impl replisdk_experimental::platform::templates::TemplateFactory for TemplateLoader {
    type Template = TeraTemplate;

    async fn load(&self, path: &std::path::Path) -> Result<Self::Template> {
        let path = path
            .to_str()
            .ok_or(TemplateError::NonUnicodePath)?;
        let tera = Tera::new(path)?;
        Ok(TeraTemplate { tera })
    }
}

/// Loaded [`tera`](tera::Tera) template.
pub struct TeraTemplate {
    /// The tera template engine to render with.
    tera: Tera,
}

impl replisdk_experimental::platform::templates::Template for TeraTemplate {
    type Output = crate::podman::Pod;

    fn render(&self, context: serde_json::Value) -> Result<Self::Output> {
        let context = tera::Context::from_value(context)?;
        let data = self.tera.render("node.yaml", &context)?;
        let pod = serde_yaml::from_str(&data)?;
        Ok(pod)
    }
}
