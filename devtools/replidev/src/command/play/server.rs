use actix_web::get;
use actix_web::App;
use actix_web::HttpServer;
use actix_web::Responder;
use anyhow::Context;
use anyhow::Result;
use replisdk_experimental::platform::templates::TemplateLookup;
use slog::Drain;

use crate::Conf;

/// HTTP Server errors.
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("unable to bind HTTP server")]
    Bind,

    #[error("HTTP server failed")]
    Failed,
}

/// Run the playground server.
pub async fn run(conf: Conf) -> Result<i32> {
    // Set up the root logger.
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = std::sync::Mutex::new(drain).fuse();
    let drain = if std::env::var("RUST_LOG").is_ok() {
        slog_envlogger::new(drain)
    } else {
        slog_envlogger::LogBuilder::new(drain)
            .filter(None, slog::FilterLevel::Info)
            .build()
    };
    let logger = slog::Logger::root(drain, slog::o!());
    let _guard = slog_scope::set_global_logger(logger.clone());
    slog_stdlog::init().expect("capture of log crate initialisation failed");

    // Load the templates manifest.
    let factory = crate::platform::TemplateLoader::default();
    let templates = TemplateLookup::load_file(factory, "stores/manifest.yaml").await?;
    let templates = actix_web::web::Data::new(templates);

    // Set up the ActixWeb server to run the Platform service.
    let bind = conf.play_server_bind.clone();
    let server = HttpServer::new(move || {
        let platform = crate::platform::Platform::from_conf(conf.clone());
        let platform = replisdk::platform::framework::into_actix_service(platform, logger.clone());
        App::new()
            .app_data(templates.clone())
            .service(index)
            .service(platform)
            .wrap(actix_web::middleware::Logger::default())
    })
    .bind(&bind)
    .context(ServerError::Bind)?
    .run();

    // Wait for the server to exit.
    println!("--> Server listening at http://{}", bind);
    server.await.context(ServerError::Failed)?;
    Ok(0)
}

#[get("/")]
async fn index() -> impl Responder {
    "Server running :-D\n".to_string()
}
