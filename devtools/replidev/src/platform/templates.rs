use anyhow::Result;
use replisdk_experimental::platform::templates::TemplateContext;
use replisdk_experimental::platform::templates::TemplateFactory;
use replisdk_experimental::platform::templates::TemplateLoadOptions;
use tera::Tera;

/// Load a [`Tera`] template.
#[derive(Default)]
pub struct TemplateLoader {}

#[async_trait::async_trait]
impl TemplateFactory for TemplateLoader {
    type Template = TeraTemplate;

    async fn load(&self, options: &TemplateLoadOptions) -> Result<Self::Template> {
        let tera = Tera::new(&options.template)?;
        let main = match options.options.get("main") {
            None => "node.yaml".to_string(),
            Some(main) if main.is_string() => main.as_str().unwrap().to_string(),
            Some(_) => {
                anyhow::bail!(anyhow::anyhow!(
                    "the main template for the node must be a string"
                ))
            }
        };
        Ok(TeraTemplate { main, tera })
    }
}

/// Loaded [`tera`](tera::Tera) template.
pub struct TeraTemplate {
    /// Name of the node template to render.
    main: String,

    /// The tera template engine to render with.
    tera: Tera,
}

impl TeraTemplate {
    /// Render a [`Pod`](crate::podman::Pod) spec from the template.
    pub fn render(&self, context: TemplateContext) -> Result<crate::podman::Pod> {
        let context = tera::Context::from_serialize(context)?;
        let data = self.tera.render(&self.main, &context)?;
        let pod = serde_yaml::from_str(&data)?;
        Ok(pod)
    }
}
