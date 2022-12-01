use actix_web::get;
use actix_web::web::Data;
use actix_web::App;
use actix_web::HttpServer;
use actix_web::Responder;
use failure::ResultExt;

use crate::Conf;
use crate::ErrorKind;
use crate::Result;

mod deprovision;
mod discover;
mod provision;

/// Run the playground server.
pub async fn run(conf: Conf) -> Result<i32> {
    let bind = conf.play_server_bind.clone();
    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(conf.clone()))
            .app_data(Data::new(discover::DiscoverData::from_conf(&conf)))
            .service(index)
            .service(discover::discover)
            .service(provision::provision)
            .service(deprovision::deprovision)
    })
    .bind(&bind)
    .with_context(|_| ErrorKind::io("http server failed to bind"))?
    .run();
    println!("--> Server listening at http://{}", bind);
    server
        .await
        .with_context(|_| ErrorKind::io("http server failed to run"))?;
    Ok(0)
}

#[get("/")]
async fn index() -> impl Responder {
    "Server running :-D\n".to_string()
}
