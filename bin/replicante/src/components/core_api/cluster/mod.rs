use slog::Logger;

use replicante_util_actixweb::RootDescriptor;

use crate::interfaces::api::APIRoot;
use crate::interfaces::api::AppConfigContext;
use crate::interfaces::Interfaces;

mod action_approve;
mod action_disapprove;
mod orchestrate;
mod orchestrator_action;
mod synthetic_view;

/// Return an `AppConfig` callback to configure cluster endpoints.
pub fn configure(logger: &Logger, interfaces: &mut Interfaces) -> impl Fn(&mut AppConfigContext) {
    let action_approve = self::action_approve::Approve::new(logger, interfaces);
    let action_disapprove = self::action_disapprove::Disapprove::new(logger, interfaces);
    let orchestrate = self::orchestrate::Orchestrate::new(logger, interfaces);
    let orchestrator_action_approve =
        self::orchestrator_action::approve::Approve::new(logger, interfaces);
    let orchestrator_action_disapprove =
        self::orchestrator_action::disapprove::Disapprove::new(logger, interfaces);
    let orchestrator_action_summary =
        self::orchestrator_action::summary::Summary::new(logger, interfaces);
    let synthetic_view = self::synthetic_view::SyntheticView::new(logger, interfaces);
    move |conf| {
        APIRoot::UnstableCoreApi.and_then(&conf.context.flags, |root| {
            let scope = actix_web::web::scope("/cluster/{cluster_id}")
                .service(action_approve.resource())
                .service(action_disapprove.resource())
                .service(orchestrate.resource())
                .service(orchestrator_action_approve.resource())
                .service(orchestrator_action_disapprove.resource())
                .service(orchestrator_action_summary.resource())
                .service(synthetic_view.resource());
            conf.scoped_service(root.prefix(), scope);
        });
    }
}
