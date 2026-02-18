//! Bot capability driven port.

use async_trait::async_trait;
use crate::FeishuError;

/// Port for querying bot identity and health.
#[async_trait]
pub trait BotInfoPort: Send + Sync {
    /// Resolve the bot's own open_id.
    async fn get_bot_open_id(&self) -> Result<Option<String>, FeishuError>;

    /// Check if the bot API is reachable and the token is valid.
    async fn health_check(&self) -> bool;
}
