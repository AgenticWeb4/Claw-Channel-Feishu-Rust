use thiserror::Error;

/// Feishu channel errors (E-FEISHU-xxxx)
#[derive(Debug, Error)]
pub enum FeishuError {
    // -- Auth (1xxx) --
    #[error("E-FEISHU-1001: failed to obtain tenant_access_token: {0}")]
    TokenFetchFailed(String),

    #[error("E-FEISHU-1002: token expired and refresh failed: {0}")]
    TokenRefreshFailed(String),

    // -- Connection (2xxx) --
    #[error("E-FEISHU-2001: WebSocket connection failed: {0}")]
    WebSocketConnectFailed(String),

    #[error("E-FEISHU-2002: WebSocket disconnected: {0}")]
    WebSocketDisconnected(String),

    // -- Message (3xxx) --
    #[error("E-FEISHU-3001: message send failed: {0}")]
    MessageSendFailed(String),

    #[error("E-FEISHU-3002: message decode failed: {0}")]
    MessageDecodeFailed(String),

    // -- Security (4xxx) --
    #[error("E-FEISHU-4001: unauthorized user: {0}")]
    UnauthorizedUser(String),

    #[error("E-FEISHU-4002: webhook signature verification failed")]
    WebhookSignatureFailed,

    // -- Bot (4.5xxx) --
    #[error("E-FEISHU-4501: bot info fetch failed: {0}")]
    BotInfoFetchFailed(String),

    // -- Capability (5xxx) --
    #[error("E-FEISHU-5001: capability start failed: {0}")]
    CapabilityStartFailed(String),

    #[error("E-FEISHU-5002: capability not found: {0}")]
    CapabilityNotFound(String),
}
