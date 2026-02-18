//! Bot adapter -- wraps open-lark's bot info API.
//!
//! Note: The Feishu bot/v3/info API returns `{"code":0, "bot":{...}}`
//! at the root level (not nested under "data"), which causes open-lark's
//! `BaseResponse::into_result()` to fail. We work around this by making
//! a direct HTTP call via the `LarkClient`'s config.

use std::sync::Arc;

use async_trait::async_trait;
use feishu_domain::{AuthPort, BotInfoPort, FeishuError};
use open_lark::client::LarkClient;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BotInfoResponse {
    code: i32,
    msg: String,
    bot: Option<BotData>,
}

#[derive(Debug, Deserialize)]
struct BotData {
    open_id: Option<String>,
}

/// Adapter for Feishu bot info using direct HTTP to work around SDK limitation.
pub struct LarkBotAdapter {
    client: Arc<LarkClient>,
    auth: Arc<dyn AuthPort>,
}

impl LarkBotAdapter {
    pub fn new(client: Arc<LarkClient>, auth: Arc<dyn AuthPort>) -> Self {
        Self { client, auth }
    }
}

#[async_trait]
impl BotInfoPort for LarkBotAdapter {
    async fn get_bot_open_id(&self) -> Result<Option<String>, FeishuError> {
        let token = self.auth.get_token().await?;

        let url = format!("{}/open-apis/bot/v3/info", self.client.config.base_url);
        let resp = self
            .client
            .config
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .map_err(|e| FeishuError::BotInfoFetchFailed(format!("request failed: {e}")))?;

        let body: BotInfoResponse = resp
            .json()
            .await
            .map_err(|e| FeishuError::BotInfoFetchFailed(format!("parse failed: {e}")))?;

        if body.code != 0 {
            return Err(FeishuError::BotInfoFetchFailed(format!(
                "API error: code={}, msg={}",
                body.code, body.msg
            )));
        }

        Ok(body.bot.and_then(|b| b.open_id))
    }

    async fn health_check(&self) -> bool {
        self.get_bot_open_id().await.is_ok()
    }
}
