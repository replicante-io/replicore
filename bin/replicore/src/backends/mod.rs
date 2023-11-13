//! Service dependency backends used in special cases of the Control Plane.
//!
//! Most service backends should be implemented in their own dedicated crates
//! and should not be defined here.
//!
//! This module is for `replicore` specific use cases that don't generalise.
//! For example the [`EventsNull`] backend for use with the `sync` command.
mod events_null;

pub use self::events_null::EventsNull;
