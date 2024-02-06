//! Asynchronous task scheduling and execution for the Replicante Control Plane.
//!
//! Task management is divided into two halves:
//!
//! - Task submission: the logic used by the control plane to request async work to be performed.
//! - Task execution: the logic that receives submitted tasks and executes the work.
//!
//! The objective of these two halves is to abstract away message queues and similar software that
//! implements the core of asynchronous task coordination and distribution.
pub mod conf;
pub mod error;
pub mod execute;
pub mod factory;
pub mod submit;

mod telemetry;
pub use self::telemetry::register_metrics;
