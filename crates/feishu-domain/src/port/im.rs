//! IM capability driven port.

use async_trait::async_trait;
use crate::FeishuError;

/// Port for sending messages through the Feishu IM API.
///
/// Note: `token` is **not** part of the interface -- open-lark's `LarkClient`
/// handles authentication internally, so the adapter never exposes raw tokens.
#[async_trait]
pub trait MessageSenderPort: Send + Sync {
    /// Send a text message to the given chat_id.
    async fn send_text(
        &self,
        chat_id: &str,
        content: &str,
    ) -> Result<(), FeishuError>;
}
