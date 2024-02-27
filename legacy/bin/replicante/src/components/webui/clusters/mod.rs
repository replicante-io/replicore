use replicante_util_actixweb::RootDescriptor;

use crate::interfaces::api::APIRoot;
use crate::interfaces::api::AppConfigContext;
use crate::Interfaces;

mod find;
mod top;

pub fn configure(interfaces: &mut Interfaces) -> impl Fn(&mut AppConfigContext) {
    let find = self::find::Find::new(interfaces);
    let top = self::top::Top::new(interfaces);
    move |conf| {
        APIRoot::UnstableWebUI.and_then(&conf.context.flags, |root| {
            let scope = actix_web::web::scope("/clusters")
                .service(find.resource_default())
                .service(find.resource_query())
                .service(top.resource());
            conf.scoped_service(root.prefix(), scope);
        });
    }
}
