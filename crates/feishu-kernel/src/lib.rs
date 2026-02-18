//! Feishu Microkernel -- Capability trait, EventBus, and runtime Kernel.
//!
//! This crate contains the microkernel runtime infrastructure.
//! Domain types (Config, Error, Models, Ports) live in `feishu-domain`.
//!
//! ```text
//!  feishu-kernel (this crate)
//! ┌────────────────────────────┐
//! │ Capability trait            │  ← plugin contract
//! │ EventBus / FeishuEvent     │  ← inter-capability communication
//! │ FeishuKernel               │  ← lifecycle manager
//! └────────────────────────────┘
//! ```

pub mod capability;
pub mod event_bus;
pub mod kernel;

pub use capability::Capability;
pub use event_bus::{EventBus, FeishuEvent};
pub use kernel::FeishuKernel;
