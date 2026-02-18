//! Auth capability driven port.

use async_trait::async_trait;
use crate::FeishuError;

/// Port for obtaining Feishu tenant_access_token.
///
/// open-lark manages token caching internally, so the adapter simply
/// delegates to `LarkClient`.  This port exists so the application layer
/// can still be tested with fakes.
#[async_trait]
pub trait AuthPort: Send + Sync {
    /// Get a valid tenant_access_token, refreshing if needed.
    async fn get_token(&self) -> Result<String, FeishuError>;

    /// Check whether the current token is expired (non-async, best-effort).
    fn is_token_expired(&self) -> bool;
}
