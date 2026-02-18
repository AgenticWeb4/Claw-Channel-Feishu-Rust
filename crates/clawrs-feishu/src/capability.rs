//! Capability wrappers -- bridge port traits to Kernel's Capability trait.
//!
//! Each wrapper allows a port adapter to participate in the Kernel's
//! lifecycle management (start/stop/health_check) without the adapter
//! itself depending on feishu-kernel.

use std::sync::Arc;

use async_trait::async_trait;
use feishu_domain::{AuthPort, BotInfoPort, FeishuError, MessageSenderPort};
use feishu_kernel::Capability;

/// Capability wrapper for auth adapters.
pub struct AuthCapability(pub Arc<dyn AuthPort>);

#[async_trait]
impl Capability for AuthCapability {
    fn name(&self) -> &str {
        "auth"
    }
    async fn start(&self) -> Result<(), FeishuError> {
        self.0
            .get_token()
            .await
            .map(|_| ())
            .map_err(|e| FeishuError::CapabilityStartFailed(format!("auth: {e}")))
    }
    async fn health_check(&self) -> bool {
        self.0.get_token().await.is_ok()
    }
}

/// Capability wrapper for bot adapters.
pub struct BotCapability(pub Arc<dyn BotInfoPort>);

#[async_trait]
impl Capability for BotCapability {
    fn name(&self) -> &str {
        "bot"
    }
    async fn start(&self) -> Result<(), FeishuError> {
        self.0
            .get_bot_open_id()
            .await
            .map(|_| ())
            .map_err(|e| FeishuError::CapabilityStartFailed(format!("bot: {e}")))
    }
    async fn health_check(&self) -> bool {
        self.0.health_check().await
    }
}

/// Capability wrapper for IM adapters.
pub struct ImCapability(#[allow(dead_code)] pub Arc<dyn MessageSenderPort>);

#[async_trait]
impl Capability for ImCapability {
    fn name(&self) -> &str {
        "im"
    }
    async fn health_check(&self) -> bool {
        true // IM is stateless; if auth is healthy, IM works
    }
}
