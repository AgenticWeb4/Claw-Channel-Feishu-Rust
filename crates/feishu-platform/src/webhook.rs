//! Webhook adapter -- HTTP server for Feishu event subscription.
//!
//! Handles URL verification (GET with challenge) and event reception (POST with encrypted JSON).
//! Feishu encrypts events with AES-256-CBC when encrypt_key is configured.

use async_trait::async_trait;
use bytes::Bytes;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use feishu_domain::codec::decode_message_content;
use feishu_domain::model::event::{FeishuEventEnvelope, FeishuMessageEvent};
use feishu_domain::{ChannelMessage, EventListenerPort, ListenEvent};
use serde::Deserialize;
use tokio::sync::mpsc;
use tracing::{error, info};

/// Feishu webhook URL verification request (GET).
#[derive(Debug, Deserialize)]
pub struct UrlVerificationRequest {
    pub challenge: Option<String>,
    #[serde(rename = "encrypt")]
    pub encrypt: Option<String>,
}

/// Feishu webhook event request (POST) - encrypted envelope.
#[derive(Debug, Deserialize)]
pub struct WebhookEventRequest {
    pub encrypt: Option<String>,
}

/// Decrypt Feishu webhook payload using encrypt_key.
/// Format: AES-256-CBC, IV = first 16 bytes of decoded content, key from encrypt_key (first 43 chars base64).
fn decrypt_webhook_payload(encrypted: &str, encrypt_key: &str) -> anyhow::Result<String> {
    use aes::cipher::{BlockDecryptMut, KeyIvInit};
    use base64::Engine;
    use cbc::cipher::block_padding::Pkcs7;

    let key_b64 = encrypt_key.chars().take(43).collect::<String>();
    let key_bytes = base64::engine::general_purpose::STANDARD
        .decode(&key_b64)
        .map_err(|e| anyhow::anyhow!("invalid encrypt_key base64: {e}"))?;
    let key: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("encrypt_key must decode to 32 bytes"))?;

    let raw = base64::engine::general_purpose::STANDARD
        .decode(encrypted)
        .map_err(|e| anyhow::anyhow!("invalid encrypted payload base64: {e}"))?;
    if raw.len() < 16 {
        anyhow::bail!("encrypted payload too short");
    }
    let (iv, ciphertext) = raw.split_at(16);
    let iv: [u8; 16] = iv.try_into().map_err(|_| anyhow::anyhow!("invalid IV"))?;

    type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;
    let dec = Aes256CbcDec::new_from_slices(&key, &iv)
        .map_err(|e| anyhow::anyhow!("cipher init failed: {e}"))?;
    let mut buf = ciphertext.to_vec();
    let decrypted = dec
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .map_err(|e| anyhow::anyhow!("decrypt failed: {e}"))?;
    String::from_utf8(decrypted.to_vec()).map_err(|e| anyhow::anyhow!("decrypted content not utf8: {e}"))
}

/// Webhook server state.
#[derive(Clone)]
struct WebhookState {
    tx: mpsc::Sender<ListenEvent>,
    encrypt_key: Option<String>,
    verification_token: Option<String>,
}

/// Adapter that runs an HTTP server for Feishu webhook events.
pub struct LarkWebhookAdapter {
    port: u16,
    encrypt_key: Option<String>,
    verification_token: Option<String>,
}

impl LarkWebhookAdapter {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            encrypt_key: None,
            verification_token: None,
        }
    }

    pub fn with_encrypt_key(mut self, key: impl Into<String>) -> Self {
        self.encrypt_key = Some(key.into());
        self
    }

    pub fn with_verification_token(mut self, token: impl Into<String>) -> Self {
        self.verification_token = Some(token.into());
        self
    }
}

#[async_trait]
impl EventListenerPort for LarkWebhookAdapter {
    async fn listen(
        &self,
        tx: mpsc::Sender<ListenEvent>,
    ) -> anyhow::Result<()> {
        let state = WebhookState {
            tx,
            encrypt_key: self.encrypt_key.clone(),
            verification_token: self.verification_token.clone(),
        };

        let app = Router::new()
            .route("/", get(handle_url_verification).post(handle_event))
            .with_state(state);

        let addr = format!("0.0.0.0:{}", self.port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        info!("Webhook server listening on http://{}", addr);
        axum::serve(listener, app).await?;
        Ok(())
    }
}

/// GET: URL verification - Feishu sends ?encrypt=xxx, we return decrypted challenge.
async fn handle_url_verification(
    State(state): State<WebhookState>,
    Query(params): Query<UrlVerificationRequest>,
) -> Response {
    let challenge = params.challenge.or_else(|| {
        params.encrypt.and_then(|enc| {
            state
                .encrypt_key
                .as_ref()
                .and_then(|key| decrypt_webhook_payload(&enc, key).ok())
        })
    });
    match challenge {
        Some(c) => Json(serde_json::json!({ "challenge": c })).into_response(),
        None => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "missing challenge or decrypt failed" })),
        )
            .into_response(),
    }
}

/// POST: Event - decrypt, parse, forward im.message.receive_v1 to tx.
async fn handle_event(
    State(state): State<WebhookState>,
    body: Bytes,
) -> Response {
    let bytes = body.as_ref();

    let json: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(j) => j,
        Err(e) => {
            error!("webhook: invalid JSON: {e}");
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "invalid json" })),
            )
                .into_response();
        }
    };

    let encrypted = json.get("encrypt").and_then(|v| v.as_str());
    let plain = match (encrypted, state.encrypt_key.as_deref()) {
        (Some(enc), Some(key)) => match decrypt_webhook_payload(enc, key) {
            Ok(p) => p,
            Err(e) => {
                error!("webhook: decrypt failed: {e}");
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": "decrypt failed" })),
                )
                    .into_response();
            }
        },
        (Some(_), None) => {
            error!("webhook: encrypted payload but no encrypt_key");
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "encrypt_key required" })),
            )
                .into_response();
        }
        (None, _) => {
            if let Some(s) = json.get("challenge").and_then(|v| v.as_str()) {
                return Json(serde_json::json!({ "challenge": s })).into_response();
            }
            json.to_string()
        }
    };

    let envelope: FeishuEventEnvelope = match serde_json::from_str(&plain) {
        Ok(e) => e,
        Err(e) => {
            error!("webhook: parse envelope failed: {e}");
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "invalid envelope" })),
            )
                .into_response();
        }
    };

    if envelope.header.event_type == "im.message.receive_v1" {
        let msg_event: FeishuMessageEvent = match serde_json::from_value(envelope.event) {
            Ok(m) => m,
            Err(e) => {
                error!("webhook: parse message event failed: {e}");
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": "invalid message event" })),
                )
                    .into_response();
            }
        };

        let sender_open_id = msg_event
            .sender
            .sender_id
            .open_id
            .clone()
            .unwrap_or_default();
        let content = decode_message_content(
            &msg_event.message.content,
            &msg_event.message.message_type,
        );
        let timestamp: u64 = 0;
        let mentioned_open_ids: Vec<String> = msg_event
            .message
            .mentions
            .as_ref()
            .map(|ms| {
                ms.iter()
                    .filter_map(|m| m.id.open_id.clone())
                    .collect()
            })
            .unwrap_or_default();

        let channel_msg = ChannelMessage {
            id: msg_event.message.message_id,
            sender: sender_open_id,
            content,
            channel: msg_event.message.chat_id,
            timestamp,
            chat_type: Some(msg_event.message.chat_type),
            mentioned_open_ids,
        };

        if state.tx.send(ListenEvent::Message(channel_msg)).await.is_err() {
            error!("webhook: failed to forward message");
        }
    }

    Json(serde_json::json!({})).into_response()
}
