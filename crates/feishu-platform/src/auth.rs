//! Auth adapter -- wraps open-lark's internal token management.

use std::sync::Arc;

use async_trait::async_trait;
use feishu_domain::{AuthPort, FeishuError};
use open_lark::client::LarkClient;

/// Adapter wrapping open-lark's `LarkClient` for auth token management.
pub struct LarkAuthAdapter {
    client: Arc<LarkClient>,
}

impl LarkAuthAdapter {
    pub fn new(client: Arc<LarkClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl AuthPort for LarkAuthAdapter {
    async fn get_token(&self) -> Result<String, FeishuError> {
        let token_manager = self.client.config.token_manager.lock().await;
        let token = token_manager
            .get_tenant_access_token(
                &self.client.config,
                "",
                "",
                &self.client.config.app_ticket_manager,
            )
            .await
            .map_err(|e| FeishuError::TokenFetchFailed(e.to_string()))?;
        Ok(token)
    }

    fn is_token_expired(&self) -> bool {
        false
    }
}
