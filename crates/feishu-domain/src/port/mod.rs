//! Driven port traits (hexagonal architecture boundary).
//!
//! These traits define the contracts that infrastructure adapters implement.
//! They live in the domain layer so the domain never depends on infrastructure.

pub mod auth;
pub mod bot;
pub mod event;
pub mod im;

pub use auth::AuthPort;
pub use bot::BotInfoPort;
pub use event::EventListenerPort;
pub use im::MessageSenderPort;
