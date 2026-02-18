//! Microkernel plugin contract.
//!
//! Each Feishu platform capability (auth, IM, event, bot, ...)
//! implements this trait and registers with the kernel.

use async_trait::async_trait;
use feishu_domain::FeishuError;

/// A Feishu platform capability (microkernel plugin).
///
/// Capabilities are started/stopped by the kernel and can report health.
/// New capabilities (cards, docs, approval, ...) are added by implementing
/// this trait -- no changes to the kernel required.
#[async_trait]
pub trait Capability: Send + Sync {
    /// Human-readable capability name (e.g. "auth", "im", "event")
    fn name(&self) -> &str;

    /// Initialize and start the capability.
    async fn start(&self) -> Result<(), FeishuError> {
        Ok(())
    }

    /// Gracefully stop the capability.
    async fn stop(&self) -> Result<(), FeishuError> {
        Ok(())
    }

    /// Health check -- returns true if the capability is operational.
    async fn health_check(&self) -> bool {
        true
    }
}
