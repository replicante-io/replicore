use slog::Logger;

use replicante_util_actixweb::RootDescriptor;

use crate::interfaces::api::APIRoot;
use crate::interfaces::api::AppConfigContext;
use crate::interfaces::Interfaces;

mod get;
mod list;

/// Return an `AppConfig` callback to configure Platform endpoints.
pub fn configure(logger: &Logger, interfaces: &mut Interfaces) -> impl Fn(&mut AppConfigContext) {
    let get = self::get::Get::new(logger, interfaces);
    let list = self::list::List::new(logger, interfaces);
    move |conf| {
        APIRoot::UnstableCoreApi.and_then(&conf.context.flags, |root| {
            let scope = actix_web::web::scope("/platforms/{namespace}").service(list.resource());
            conf.scoped_service(root.prefix(), scope);
            let scope = actix_web::web::scope("/platform/{namespace}").service(get.resource());
            conf.scoped_service(root.prefix(), scope);
        });
    }
}
