//! Format output for easy consumption by people interacting with `replictl`.
use super::ops::Ops;
use super::ops::Responses;
use super::FormatterStrategy;
use crate::globals::Globals;

mod context;

/// Format output for easy consumption by people interacting with `replictl`.
pub struct HumanFormatter;

impl FormatterStrategy for HumanFormatter {
    fn format(&self, _: &Globals, op: Ops) -> Responses {
        match op {
            Ops::Context(context) => {
                self::context::show(&context);
                Responses::Success
            }
            Ops::ContextList => Responses::from(self::context::ContextList::new()),
        }
    }
}
