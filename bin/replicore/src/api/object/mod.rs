//! API endpoints for handling persisted objects.
use actix_web::web::ServiceConfig;

pub mod namespace;

/// Configure all API endpoints defined in this module.
pub fn configure(config: &mut ServiceConfig) {
    config
        .service(self::namespace::delete)
        .service(self::namespace::get)
        .service(self::namespace::list);
}
