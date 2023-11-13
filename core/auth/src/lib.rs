//! Authentication and Authorisation data and interfaces for the Replicante Core Control Plane.
//!
//! First of all:
//!
//! - Authentication: answers "who is asking for access?" (it is about identity).
//! - Authorisation: answers "can they do what they are asking to do?" (it is about access).
//!
//! For more details on how [`Entity`]s, [`Action`]s and [`Resource`]s are modelled refer to
//! the [`replisdk::core::models::auth`] module.
//!
//! ## Impersonation
//!
//! Impersonation enables one entity to act as another.
//!
//! Aside from user oriented reasons to have impersonation, it enables another key feature:
//! performing system operations with scoped permissions.
//! For example, impersonation can be used by the cluster orchestration logic to act with
//! restricted access to other resources instead of having unlimited access.
//!
//! When impersonation is involved authentication and authorisation work as follow:
//!
//! 1. Authenticate the [`Entity`] requesting the action.
//! 2. Check the [`Entity`] is authorised to impersonate the [`ImpersonateEntity`]:
//!    - Action: `auth:impersonate`.
//!    - Resource: the target [`ImpersonateEntity`].
//! 3. Check the [`ImpersonateEntity`] is authorised to perform the request.
pub mod access;
pub mod identity;

// Re-export replisdk definitions for convenience.
pub use replisdk::core::models::auth::Action;
pub use replisdk::core::models::auth::AuthContext;
pub use replisdk::core::models::auth::Entity;
pub use replisdk::core::models::auth::EntityService;
pub use replisdk::core::models::auth::EntitySystem;
pub use replisdk::core::models::auth::EntityUser;
pub use replisdk::core::models::auth::ImpersonateEntity;
pub use replisdk::core::models::auth::Resource;
pub use replisdk::core::models::auth::RESOURCE_NAMESPACE;
