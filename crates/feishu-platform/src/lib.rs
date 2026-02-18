//! Feishu Platform -- adapters wrapping open-lark SDK.
//!
//! Consolidates Auth, IM, Bot, and Event adapters in one crate.
//! Port traits remain in feishu-domain; this crate provides implementations.

pub mod auth;
pub mod bot;
pub mod event;
pub mod im;

pub use auth::LarkAuthAdapter;
pub use bot::LarkBotAdapter;
pub use event::LarkWsAdapter;
pub use im::LarkImAdapter;
