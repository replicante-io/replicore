//! API endpoints for handling persisted objects.
use actix_web::web::ServiceConfig;

pub mod cluster_spec;
pub mod namespace;
pub mod platform;

/// Configure all API endpoints defined in this module.
pub fn configure(config: &mut ServiceConfig) {
    config
        .service(self::cluster_spec::delete)
        .service(self::cluster_spec::get)
        .service(self::cluster_spec::list)
        .service(self::namespace::delete)
        .service(self::namespace::get)
        .service(self::namespace::list)
        .service(self::platform::delete)
        .service(self::platform::discover)
        .service(self::platform::get)
        .service(self::platform::list);
}
