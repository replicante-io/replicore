use actix_web::get;
use actix_web::App;
use actix_web::HttpServer;
use actix_web::Responder;
use anyhow::Context;
use anyhow::Result;
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
    let logger = slog::Logger::root(drain, slog::o!());

    // Set up the ActixWeb server to run the Platform service.
    let bind = conf.play_server_bind.clone();
    let server = HttpServer::new(move || {
        let platform = crate::platform::Platform::from_conf(conf.clone());
        let platform = replisdk::platform::framework::into_actix_service(platform, logger.clone());

        App::new()
            .service(index)
            .service(platform)
    })
    .bind(&bind)
    .context(ServerError::Bind)?
    .run();

    // Wait for the server to exit.
    println!("--> Server listening at http://{}", bind);
    server
        .await
        .context(ServerError::Failed)?;
    Ok(0)
}

#[get("/")]
async fn index() -> impl Responder {
    "Server running :-D\n".to_string()
}
