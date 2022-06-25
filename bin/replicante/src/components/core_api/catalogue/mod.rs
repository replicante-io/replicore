use slog::Logger;

use replicante_util_actixweb::RootDescriptor;

use crate::interfaces::api::APIRoot;
use crate::interfaces::api::AppConfigContext;
use crate::interfaces::Interfaces;

mod orchestrator_actions;

/// Return an `AppConfig` callback to configure Catalogue endpoints.
pub fn configure(logger: &Logger, interfaces: &mut Interfaces) -> impl Fn(&mut AppConfigContext) {
    let orchestrator_actions =
        self::orchestrator_actions::OrchestratorActions::new(logger, interfaces);
    move |conf| {
        APIRoot::UnstableCatalogue.and_then(&conf.context.flags, |root| {
            conf.scoped_service(root.prefix(), orchestrator_actions.resource());
        });
    }
}
