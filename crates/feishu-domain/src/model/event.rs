//! Feishu event value objects -- deserialized from WebSocket/webhook JSON payloads.

use serde::Deserialize;

/// Feishu sender ID (multi-type identity)
#[derive(Debug, Clone, Deserialize)]
pub struct FeishuSenderId {
    pub open_id: Option<String>,
    pub user_id: Option<String>,
    pub union_id: Option<String>,
}

/// Feishu message mention entry
#[derive(Debug, Clone, Deserialize)]
pub struct FeishuMention {
    pub key: String,
    pub id: FeishuSenderId,
    pub name: String,
}

/// Feishu event envelope (schema 2.0)
#[derive(Debug, Deserialize)]
pub struct FeishuEventEnvelope {
    #[allow(dead_code)]
    pub schema: Option<String>,
    pub header: FeishuEventHeader,
    pub event: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct FeishuEventHeader {
    #[allow(dead_code)]
    pub event_id: String,
    pub event_type: String,
    #[allow(dead_code)]
    pub create_time: String,
    #[allow(dead_code)]
    pub token: String,
    #[allow(dead_code)]
    pub app_id: String,
    #[allow(dead_code)]
    pub tenant_key: String,
}

/// Feishu message event (im.message.receive_v1)
#[derive(Debug, Deserialize)]
pub struct FeishuMessageEvent {
    pub sender: FeishuSender,
    pub message: FeishuMessageBody,
}

#[derive(Debug, Deserialize)]
pub struct FeishuSender {
    pub sender_id: FeishuSenderId,
    #[allow(dead_code)]
    pub sender_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FeishuMessageBody {
    #[allow(dead_code)]
    pub message_id: String,
    pub chat_id: String,
    pub chat_type: String,
    pub message_type: String,
    pub content: String,
    pub mentions: Option<Vec<FeishuMention>>,
    #[allow(dead_code)]
    pub root_id: Option<String>,
    #[allow(dead_code)]
    pub parent_id: Option<String>,
}
