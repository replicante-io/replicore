use slog::Logger;

use replicante_util_actixweb::RootDescriptor;

use crate::interfaces::api::APIRoot;
use crate::interfaces::api::AppConfigContext;
use crate::interfaces::Interfaces;

mod node_action;
mod orchestrate;
mod orchestrator_action;
mod synthetic_view;

/// Return an `AppConfig` callback to configure cluster endpoints.
pub fn configure(logger: &Logger, interfaces: &mut Interfaces) -> impl Fn(&mut AppConfigContext) {
    let node_action_approve = self::node_action::approve::Approve::new(logger, interfaces);
    let node_action_disapprove = self::node_action::disapprove::Disapprove::new(logger, interfaces);
    let node_action_summary = self::node_action::summary::Summary::new(logger, interfaces);
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
                .service(node_action_approve.resource())
                .service(node_action_disapprove.resource())
                .service(node_action_summary.resource())
                .service(orchestrate.resource())
                .service(orchestrator_action_approve.resource())
                .service(orchestrator_action_disapprove.resource())
                .service(orchestrator_action_summary.resource())
                .service(synthetic_view.resource());
            conf.scoped_service(root.prefix(), scope);
        });
    }
}
