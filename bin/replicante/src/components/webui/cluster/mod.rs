use replicante_util_actixweb::RootDescriptor;

use crate::interfaces::api::APIRoot;
use crate::interfaces::api::AppConfigContext;
use crate::Interfaces;

mod actions;
mod agents;
mod discovery;
mod events;
mod meta;
mod nodes;

pub fn configure(interfaces: &mut Interfaces) -> impl Fn(&mut AppConfigContext) {
    let action = self::actions::ActionInfo::new(interfaces);
    let actions = self::actions::Actions::new(interfaces);
    let agents = self::agents::Agents::new(interfaces);
    let discovery = self::discovery::Discovery::new(interfaces);
    let events = self::events::Events::new(interfaces);
    let meta = self::meta::Meta::new(interfaces);
    let nodes = self::nodes::Nodes::new(interfaces);
    move |conf| {
        APIRoot::UnstableWebUI.and_then(&conf.context.flags, |root| {
            let scope = actix_web::web::scope("/cluster/{cluster_id}")
                .service(action.resource())
                .service(actions.resource())
                .service(agents.resource())
                .service(discovery.resource())
                .service(events.resource())
                .service(meta.resource())
                .service(nodes.resource());
            conf.scoped_service(root.prefix(), scope);
        });
    }
}
