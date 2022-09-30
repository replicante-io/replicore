use std::sync::Arc;

use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use failure::ResultExt;
use slog::Logger;

use replicante_store_view::store::events::EventsFilters;
use replicante_store_view::store::events::EventsOptions;
use replicante_store_view::store::Store;
use replicante_util_actixweb::with_request_span;
use replicante_util_actixweb::RootDescriptor;
use replicante_util_actixweb::TracingMiddleware;

use super::constants::RECENT_EVENTS_LIMIT;
use crate::interfaces::api::APIRoot;
use crate::interfaces::api::AppConfigContext;
use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

pub fn configure(interfaces: &mut Interfaces) -> impl Fn(&mut AppConfigContext) {
    let events = Events::new(interfaces);
    move |conf| {
        APIRoot::UnstableWebUI.and_then(&conf.context.flags, |root| {
            conf.scoped_service(root.prefix(), events.resource());
        });
    }
}

struct Events {
    data: web::Data<EventsData>,
    logger: Logger,
    tracer: Arc<opentracingrust::Tracer>,
}

impl Events {
    pub fn new(interfaces: &mut Interfaces) -> Self {
        let data = EventsData {
            store: interfaces.stores.view.clone(),
        };
        Events {
            data: web::Data::new(data),
            logger: interfaces.logger.clone(),
            tracer: interfaces.tracing.tracer(),
        }
    }

    pub fn resource(&self) -> impl HttpServiceFactory {
        let logger = self.logger.clone();
        let tracer = Arc::clone(&self.tracer);
        let tracer = TracingMiddleware::with_name(logger, tracer, "/events");
        web::resource("/events")
            .app_data(self.data.clone())
            .wrap(tracer)
            .route(web::get().to(responder))
    }
}

#[derive(Clone)]
struct EventsData {
    store: Store,
}

async fn responder(data: web::Data<EventsData>, request: HttpRequest) -> Result<impl Responder> {
    let options = EventsOptions {
        limit: Some(RECENT_EVENTS_LIMIT),
        reverse: true,
    };
    let mut request = request;
    let iter = with_request_span(&mut request, |span| {
        let span = span.map(|span| span.context().clone());
        data.store
            .events()
            .range(EventsFilters::all(), options, span)
            .with_context(|_| ErrorKind::PrimaryStoreQuery("events"))
    })?;
    let mut events = Vec::new();
    for event in iter {
        let event = event.with_context(|_| ErrorKind::Deserialize("event record", "Event"))?;
        events.push(event);
    }

    let response = HttpResponse::Ok().json(events);
    Ok(response)
}
