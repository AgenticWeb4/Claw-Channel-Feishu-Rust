//! Node.js bindings for clawrs-feishu via napi-rs.
//!
//! Exports: createChannel, FeishuChannel (send, healthCheck, listen).

#![allow(clippy::needless_return)]

use std::sync::Arc;

use napi_derive::napi;

use clawrs_feishu::{
    create_channel, Channel, ChannelMessage, FeishuConfig, FeishuConnectionMode,
    FeishuDomain,
};
use feishu_domain::{DmPolicy, GroupPolicy};
use napi::bindgen_prelude::*;
use napi::threadsafe_function::{
    ErrorStrategy, ThreadSafeCallContext, ThreadsafeFunction,
    ThreadsafeFunctionCallMode,
};
use napi::JsFunction;
use tokio::sync::mpsc;

/// JS config input (OpenClaw-compatible field names).
#[napi(object)]
pub struct FeishuConfigInput {
    pub app_id: String,
    pub app_secret: String,
    pub domain: Option<String>,
    pub connection_mode: Option<String>,
    pub allowed_users: Option<Vec<String>>,
    pub dm_policy: Option<String>,
    pub group_policy: Option<String>,
    pub allow_from: Option<Vec<String>>,
    pub group_allow_from: Option<Vec<String>>,
    pub group_require_mention: Option<bool>,
    pub encrypt_key: Option<String>,
    pub verification_token: Option<String>,
    pub webhook_port: Option<u16>,
}

fn parse_dm_policy(s: &str) -> DmPolicy {
    match s.to_lowercase().as_str() {
        "open" => DmPolicy::Open,
        "deny" => DmPolicy::Deny,
        _ => DmPolicy::Pairing,
    }
}

fn parse_group_policy(s: &str) -> GroupPolicy {
    match s.to_lowercase().as_str() {
        "open" => GroupPolicy::Open,
        "deny" => GroupPolicy::Deny,
        _ => GroupPolicy::Allowlist,
    }
}

fn to_feishu_config(input: FeishuConfigInput) -> FeishuConfig {
    let domain = input
        .domain
        .as_deref()
        .map(|s| match s.to_lowercase().as_str() {
            "lark" => FeishuDomain::Lark,
            _ => FeishuDomain::Feishu,
        })
        .unwrap_or_default();

    let connection_mode = input
        .connection_mode
        .as_deref()
        .map(|s| match s.to_lowercase().as_str() {
            "webhook" => FeishuConnectionMode::Webhook,
            _ => FeishuConnectionMode::WebSocket,
        })
        .unwrap_or_default();

    let dm_policy = input
        .dm_policy
        .as_deref()
        .map(parse_dm_policy);

    let group_policy = input
        .group_policy
        .as_deref()
        .map(parse_group_policy);

    FeishuConfig {
        app_id: input.app_id,
        app_secret: input.app_secret,
        domain,
        connection_mode,
        // BUG FIX: default to ["*"] (allow all) instead of [] (deny all).
        // Empty allowed_users -> DmPolicy::from_allowlist(&[]) -> Deny,
        // which silently drops ALL DM messages.
        allowed_users: input.allowed_users.unwrap_or_else(|| vec!["*".to_string()]),
        dm_policy,
        group_policy,
        allow_from: input.allow_from,
        group_allow_from: input.group_allow_from.unwrap_or_default(),
        group_require_mention: input.group_require_mention.unwrap_or(true),
        encrypt_key: input.encrypt_key,
        verification_token: input.verification_token,
        webhook_port: input.webhook_port,
    }
}

/// Create a Feishu channel from config.
#[napi]
pub fn create_channel_js(config: FeishuConfigInput) -> Result<FeishuChannelJs> {
    let rust_config = to_feishu_config(config);
    let channel = create_channel(rust_config);
    Ok(FeishuChannelJs {
        inner: Arc::new(channel),
    })
}

/// Health check (standalone, for testing).
#[napi]
pub fn health_check_standalone() -> bool {
    true
}

/// Feishu channel wrapper for Node.js.
#[napi]
pub struct FeishuChannelJs {
    inner: Arc<dyn Channel + Send + Sync>,
}

#[napi]
impl FeishuChannelJs {
    /// Send a text message to a chat.
    #[napi]
    pub async fn send(&self, message: String, recipient: String) -> Result<()> {
        self.inner
            .send(&message, &recipient)
            .await
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Run health check.
    #[napi]
    pub async fn health_check(&self) -> bool {
        self.inner.health_check().await
    }

    /// Start listening; invokes `on_message` for each received message (JSON string).
    ///
    /// Connection lifecycle signals are sent as special JSON messages:
    ///   - `{"__signal__":"connected"}` when WebSocket connects
    ///   - `{"__signal__":"disconnected","error":"..."}` on disconnect/error
    ///   - `{"__signal__":"error","error":"..."}` on fatal error
    ///
    /// Returns when the listener has started. Listening runs in background.
    #[napi]
    pub fn listen(&self, on_message: JsFunction) -> Result<()> {
        let tsfn: ThreadsafeFunction<String, ErrorStrategy::Fatal> =
            on_message.create_threadsafe_function(
                0,
                |ctx: ThreadSafeCallContext<String>| Ok(vec![ctx.value]),
            )?;

        let (tx, mut rx) = mpsc::channel::<ChannelMessage>(256);
        let inner = self.inner.clone();

        // Clone tsfn for error notification from listen task
        let error_tsfn = tsfn.clone();

        let _listen_handle = tokio::spawn(async move {
            tracing::info!("clawrs-feishu: starting Rust listen task");

            match inner.listen(tx).await {
                Ok(()) => {
                    tracing::info!("clawrs-feishu: listen completed normally");
                    let signal = r#"{"__signal__":"disconnected","error":"connection closed normally"}"#.to_string();
                    let _ = error_tsfn.call(signal, ThreadsafeFunctionCallMode::NonBlocking);
                }
                Err(e) => {
                    tracing::error!("clawrs-feishu listen error: {e}");
                    let err_escaped = e.to_string().replace('"', "\\\"");
                    let signal = format!(r#"{{"__signal__":"error","error":"{}"}}"#, err_escaped);
                    let _ = error_tsfn.call(signal, ThreadsafeFunctionCallMode::NonBlocking);
                }
            }
        });

        // Clone tsfn for the connected signal
        let connected_tsfn = tsfn.clone();

        tokio::spawn(async move {
            let mut connected_signaled = false;

            while let Some(msg) = rx.recv().await {
                // Send connection signal on first message
                if !connected_signaled {
                    connected_signaled = true;
                    let signal = r#"{"__signal__":"connected"}"#.to_string();
                    let _ = connected_tsfn.call(signal, ThreadsafeFunctionCallMode::NonBlocking);
                }

                let js_msg = ChannelMessageJs {
                    id: msg.id,
                    sender: msg.sender,
                    content: msg.content,
                    channel: msg.channel,
                    timestamp: msg.timestamp as f64,
                    chat_type: msg.chat_type,
                    mentioned_open_ids: msg.mentioned_open_ids,
                };
                if let Ok(json) = serde_json::to_string(&js_msg) {
                    let _ = tsfn.call(json, ThreadsafeFunctionCallMode::Blocking);
                }
            }
            tsfn.abort().ok();
        });

        Ok(())
    }
}

/// Message received from the channel (passed as JSON string to callback).
#[derive(serde::Serialize)]
pub struct ChannelMessageJs {
    pub id: String,
    pub sender: String,
    pub content: String,
    pub channel: String,
    pub timestamp: f64,
    pub chat_type: Option<String>,
    pub mentioned_open_ids: Vec<String>,
}
