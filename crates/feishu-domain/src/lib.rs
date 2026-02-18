//! Feishu Domain -- pure domain types with zero infrastructure dependencies.
//!
//! This crate defines the shared vocabulary of the entire Feishu channel:
//! - Configuration value objects
//! - Domain errors
//! - Domain models (ChannelMessage, Feishu event envelopes)
//! - Security domain services (SecurityGuard, policies)
//! - Port traits (driven ports for hexagonal architecture)
//! - Message codec (pure encoding/decoding functions)
//!
//! ```text
//!  feishu-domain (this crate)
//! ┌──────────────────────────────────────┐
//! │ config    ── FeishuConfig, Domain    │
//! │ error     ── FeishuError             │
//! │ model/    ── ChannelMessage, events  │
//! │ security/ ── SecurityGuard, policies │
//! │ port/     ── AuthPort, BotInfoPort,  │
//! │              MessageSenderPort,      │
//! │              EventListenerPort       │
//! │ codec     ── encode / decode / strip │
//! └──────────────────────────────────────┘
//! ```

pub mod codec;
pub mod config;
pub mod error;
pub mod model;
pub mod port;
pub mod security;

// -- Ergonomic re-exports --

pub use config::{FeishuConfig, FeishuConnectionMode, FeishuDomain};
pub use error::FeishuError;
pub use model::ChannelMessage;
pub use port::{AuthPort, BotInfoPort, EventListenerPort, MessageSenderPort};
pub use security::{DmPolicy, GroupPolicy, SecurityGuard};
