//! IM adapter -- wraps open-lark's message sending API.

use std::sync::Arc;

use async_trait::async_trait;
use feishu_domain::codec::encode_text_message;
use feishu_domain::{FeishuError, MessageSenderPort};
use open_lark::client::LarkClient;
use open_lark::service::im::v1::message::{
    CreateMessageRequest, CreateMessageRequestBody,
};

/// Adapter wrapping open-lark's IM message create API.
pub struct LarkImAdapter {
    client: Arc<LarkClient>,
}

impl LarkImAdapter {
    pub fn new(client: Arc<LarkClient>) -> Self {
        Self { client }
    }
}

/// Infer the Feishu `receive_id_type` from the ID prefix.
///
/// - `oc_` → `chat_id`  (group chat)
/// - `ou_` → `open_id`  (user open id)
/// - `on_` → `union_id` (cross-app user id)
/// - other → `open_id`  (safest default)
fn infer_receive_id_type(id: &str) -> &'static str {
    if id.starts_with("oc_") {
        "chat_id"
    } else if id.starts_with("on_") {
        "union_id"
    } else {
        "open_id"
    }
}

#[async_trait]
impl MessageSenderPort for LarkImAdapter {
    async fn send_text(
        &self,
        receive_id: &str,
        content: &str,
    ) -> Result<(), FeishuError> {
        let id_type = infer_receive_id_type(receive_id);

        let body = CreateMessageRequestBody::builder()
            .receive_id(receive_id)
            .msg_type("text")
            .content(encode_text_message(content))
            .build();

        let request = CreateMessageRequest::builder()
            .receive_id_type(id_type)
            .request_body(body)
            .build();

        let _msg = self
            .client
            .im
            .v1
            .message
            .create(request, None)
            .await
            .map_err(|e| FeishuError::MessageSendFailed(e.to_string()))?;

        Ok(())
    }
}
