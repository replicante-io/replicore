use slog::Logger;

use replicante_util_actixweb::RootDescriptor;

use crate::interfaces::api::APIRoot;
use crate::interfaces::api::AppConfigContext;
use crate::interfaces::Interfaces;

mod delete;
mod list;

/// Return an `AppConfig` callback to configure DiscoverySettings endpoints.
pub fn configure(logger: &Logger, interfaces: &mut Interfaces) -> impl Fn(&mut AppConfigContext) {
    let delete = self::delete::Delete::new(logger, interfaces);
    let list = self::list::List::new(logger, interfaces);
    move |conf| {
        APIRoot::UnstableCoreApi.and_then(&conf.context.flags, |root| {
            let scope = actix_web::web::scope("/discoverysettings/{namespace}")
                .service(delete.resource())
                .service(list.resource());
            conf.scoped_service(root.prefix(), scope);
        });
    }
}
