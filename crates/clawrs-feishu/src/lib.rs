//! Claw Feishu Channel -- facade crate (composition root).
//!
//! Wires together the 4-crate architecture for zeroclaw, openclaw, and other Claw-family products.
//!
//! ```text
//!  ┌─────────────────────────────────────────────────────────────────┐
//!  │  clawrs-feishu (this crate)                                    │
//!  │                                                                  │
//!  │  compose::create_channel(FeishuConfig)                          │
//!  │    ├─ feishu-platform ── LarkAuthAdapter, LarkImAdapter,        │
//!  │    │                      LarkBotAdapter, LarkWsAdapter         │
//!  │    ├─ feishu-kernel    ── FeishuKernel + EventBus               │
//!  │    └─ feishu-domain   ── ports + models + security              │
//!  │                                                                  │
//!  │  channel::FeishuChannelService ── Channel trait impl             │
//!  │  capability:: ── AuthCapability, BotCapability, ImCapability     │
//!  └─────────────────────────────────────────────────────────────────┘
//! ```

pub mod capability;
pub mod channel;
pub mod compose;

// -- Ergonomic re-exports for library consumers --

pub use feishu_domain::{
    AuthPort, BotInfoPort, ChannelMessage, EventListenerPort, FeishuConfig,
    FeishuConnectionMode, FeishuDomain, FeishuError, MessageSenderPort,
    SecurityGuard,
};
pub use feishu_kernel::{Capability, EventBus, FeishuEvent, FeishuKernel};

pub use channel::{Channel, FeishuChannelService};
pub use compose::create_channel;
