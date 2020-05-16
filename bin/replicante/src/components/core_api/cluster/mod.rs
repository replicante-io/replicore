use slog::Logger;

use replicante_util_actixweb::RootDescriptor;

use crate::interfaces::api::APIRoot;
use crate::interfaces::api::AppConfigContext;
use crate::interfaces::Interfaces;

mod action_approve;
mod action_disapprove;
mod refresh;

/// Return an `AppConfig` callback to configure cluster endpoints.
pub fn configure(logger: &Logger, interfaces: &mut Interfaces) -> impl Fn(&mut AppConfigContext) {
    let approve = self::action_approve::Approve::new(logger, interfaces);
    let disapprove = self::action_disapprove::Disapprove::new(logger, interfaces);
    let refresh = self::refresh::Refresh::new(logger, interfaces);
    move |conf| {
        APIRoot::UnstableCoreApi.and_then(&conf.context.flags, |root| {
            let scope = actix_web::web::scope("/cluster/{cluster_id}")
                .service(approve.resource())
                .service(disapprove.resource())
                .service(refresh.resource());
            conf.scoped_service(root.prefix(), scope);
        });
    }
}
