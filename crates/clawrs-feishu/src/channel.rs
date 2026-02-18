//! Channel trait and FeishuChannelService (aggregate root).
//!
//! Follows Clean Architecture: Channel is the driving port; FeishuChannelService
//! is the aggregate root receiving adapters via constructor injection.

use std::sync::Arc;

use async_trait::async_trait;
use feishu_domain::{
    AuthPort, BotInfoPort, ChannelMessage, EventListenerPort, FeishuConfig,
    FeishuConnectionMode, MessageSenderPort, SecurityGuard,
};
use feishu_kernel::FeishuKernel;

mod filter;

/// Core channel trait -- driving port.
///
/// Mirrors `zeroclaw_core::channels::traits::Channel` for independent compilation.
#[async_trait]
pub trait Channel: Send + Sync {
    fn name(&self) -> &str;
    async fn send(&self, message: &str, recipient: &str) -> anyhow::Result<()>;
    async fn listen(
        &self,
        tx: tokio::sync::mpsc::Sender<ChannelMessage>,
    ) -> anyhow::Result<()>;
    async fn health_check(&self) -> bool {
        true
    }
}

/// Feishu channel service -- the aggregate root.
///
/// Receives capability adapters via constructor injection. Never knows
/// about HTTP, WebSocket, or any other infrastructure detail.
pub struct FeishuChannelService {
    config: FeishuConfig,
    #[allow(dead_code)]
    auth: Arc<dyn AuthPort>,
    sender: Arc<dyn MessageSenderPort>,
    bot_info: Arc<dyn BotInfoPort>,
    events: Arc<dyn EventListenerPort>,
    security: SecurityGuard,
    kernel: FeishuKernel,
}

impl FeishuChannelService {
    pub fn new(
        config: FeishuConfig,
        auth: Arc<dyn AuthPort>,
        sender: Arc<dyn MessageSenderPort>,
        bot_info: Arc<dyn BotInfoPort>,
        events: Arc<dyn EventListenerPort>,
        security: SecurityGuard,
        kernel: FeishuKernel,
    ) -> Self {
        Self {
            config,
            auth,
            sender,
            bot_info,
            events,
            security,
            kernel,
        }
    }

    pub fn security(&self) -> &SecurityGuard {
        &self.security
    }

    pub fn config(&self) -> &FeishuConfig {
        &self.config
    }

    pub fn kernel(&self) -> &FeishuKernel {
        &self.kernel
    }
}

#[async_trait]
impl Channel for FeishuChannelService {
    fn name(&self) -> &str {
        "feishu"
    }

    async fn send(&self, message: &str, chat_id: &str) -> anyhow::Result<()> {
        self.sender
            .send_text(chat_id, message)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(())
    }

    /// Start listening with application-level security + @mention filtering.
    ///
    /// Flow:
    /// 1. Resolve bot_open_id for group-chat mention detection
    /// 2. Spawn filter task: raw events -> SecurityGuard -> @mention -> user tx
    /// 3. Delegate to EventListenerPort (open-lark WebSocket with reconnection)
    async fn listen(
        &self,
        tx: tokio::sync::mpsc::Sender<ChannelMessage>,
    ) -> anyhow::Result<()> {
        tracing::info!("Feishu channel: starting listener");

        if matches!(self.config.connection_mode, FeishuConnectionMode::Webhook) {
            anyhow::bail!("Webhook mode not yet implemented -- use websocket mode");
        }

        let bot_open_id = match self.bot_info.get_bot_open_id().await {
            Ok(id) => {
                if let Some(ref oid) = id {
                    tracing::info!("Feishu channel: bot open_id = {oid}");
                }
                id
            }
            Err(e) => {
                tracing::warn!("Feishu channel: could not resolve bot open_id: {e}");
                None
            }
        };

        let (internal_tx, mut internal_rx) =
            tokio::sync::mpsc::channel::<ChannelMessage>(256);

        let filter_params = filter::FilterParams {
            security: self.security.clone(),
            bot_open_id: bot_open_id.clone(),
            group_require_mention: self.config.group_require_mention,
        };
        let user_tx = tx;

        let filter_handle = tokio::spawn(async move {
            while let Some(msg) = internal_rx.recv().await {
                if !filter::should_forward_message(&msg, &filter_params) {
                    continue;
                }
                if let Err(e) = user_tx.send(msg).await {
                    tracing::error!("Feishu: failed to forward message: {e}");
                    break;
                }
            }
        });

        let result = self.events.listen(internal_tx).await;
        filter_handle.abort();
        result
    }

    /// Health check via Kernel (aggregates all capability checks).
    async fn health_check(&self) -> bool {
        let results = self.kernel.health_check_all().await;
        let all_healthy = results.iter().all(|(_, h)| *h);
        if !all_healthy {
            for (name, healthy) in &results {
                if !healthy {
                    tracing::warn!("Feishu: capability '{name}' is unhealthy");
                }
            }
        }
        all_healthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::{AuthCapability, BotCapability, ImCapability};
    use feishu_domain::{FeishuDomain, FeishuError};
    use feishu_kernel::EventBus;

    struct FakeAuth;
    #[async_trait]
    impl AuthPort for FakeAuth {
        async fn get_token(&self) -> Result<String, FeishuError> {
            Ok("fake_token".into())
        }
        fn is_token_expired(&self) -> bool {
            false
        }
    }

    struct FakeSender;
    #[async_trait]
    impl MessageSenderPort for FakeSender {
        async fn send_text(&self, _: &str, _: &str) -> Result<(), FeishuError> {
            Ok(())
        }
    }

    struct FakeBotInfo;
    #[async_trait]
    impl BotInfoPort for FakeBotInfo {
        async fn get_bot_open_id(&self) -> Result<Option<String>, FeishuError> {
            Ok(Some("ou_fake_bot".into()))
        }
        async fn health_check(&self) -> bool {
            true
        }
    }

    struct FakeEvents;
    #[async_trait]
    impl EventListenerPort for FakeEvents {
        async fn listen(
            &self,
            _tx: tokio::sync::mpsc::Sender<ChannelMessage>,
        ) -> anyhow::Result<()> {
            Ok(())
        }
    }

    fn test_config() -> FeishuConfig {
        FeishuConfig {
            app_id: "test".into(),
            app_secret: "secret".into(),
            domain: FeishuDomain::default(),
            connection_mode: FeishuConnectionMode::default(),
            allowed_users: vec!["*".into()],
            dm_policy: None,
            group_policy: None,
            allow_from: None,
            group_allow_from: vec![],
            group_require_mention: true,
            encrypt_key: None,
            verification_token: None,
            webhook_port: None,
        }
    }

    fn test_service() -> FeishuChannelService {
        let auth: Arc<dyn AuthPort> = Arc::new(FakeAuth);
        let sender: Arc<dyn MessageSenderPort> = Arc::new(FakeSender);
        let bot_info: Arc<dyn BotInfoPort> = Arc::new(FakeBotInfo);
        let events: Arc<dyn EventListenerPort> = Arc::new(FakeEvents);
        let security = SecurityGuard::new(vec!["*".into()]);

        let mut kernel = FeishuKernel::new(EventBus::default());
        kernel.register(Arc::new(AuthCapability(auth.clone())));
        kernel.register(Arc::new(BotCapability(bot_info.clone())));
        kernel.register(Arc::new(ImCapability(sender.clone())));

        FeishuChannelService::new(
            test_config(),
            auth,
            sender,
            bot_info,
            events,
            security,
            kernel,
        )
    }

    #[test]
    fn channel_name_is_feishu() {
        let svc = test_service();
        assert_eq!(svc.name(), "feishu");
    }

    #[tokio::test]
    async fn send_succeeds_with_fake_ports() {
        let svc = test_service();
        assert!(svc.send("hello", "oc_test").await.is_ok());
    }

    #[tokio::test]
    async fn health_check_via_kernel() {
        let svc = test_service();
        assert!(svc.health_check().await);
        assert_eq!(svc.kernel().capability_count(), 3);
    }

    #[tokio::test]
    async fn listen_succeeds_with_fake_events() {
        let svc = test_service();
        let (tx, _rx) = tokio::sync::mpsc::channel(10);
        assert!(svc.listen(tx).await.is_ok());
    }

    #[test]
    fn security_guard_exposed() {
        let svc = test_service();
        assert!(svc.security().is_user_allowed("anyone"));
    }

    #[tokio::test]
    async fn kernel_start_all_with_fakes() {
        let svc = test_service();
        assert!(svc.kernel().start_all().await.is_ok());
    }

    #[tokio::test]
    async fn listen_filters_unauthorized_user() {
        let auth: Arc<dyn AuthPort> = Arc::new(FakeAuth);
        let sender: Arc<dyn MessageSenderPort> = Arc::new(FakeSender);
        let bot_info: Arc<dyn BotInfoPort> = Arc::new(FakeBotInfo);
        let security = SecurityGuard::new(vec!["ou_allowed".into()]);

        struct FilterTestEvents;
        #[async_trait]
        impl EventListenerPort for FilterTestEvents {
            async fn listen(
                &self,
                tx: tokio::sync::mpsc::Sender<ChannelMessage>,
            ) -> anyhow::Result<()> {
                tx.send(ChannelMessage {
                    id: "m1".into(),
                    sender: "ou_allowed".into(),
                    content: "hello".into(),
                    channel: "feishu".into(),
                    timestamp: 0,
                    chat_type: Some("p2p".into()),
                    mentioned_open_ids: vec![],
                })
                .await
                .ok();
                tx.send(ChannelMessage {
                    id: "m2".into(),
                    sender: "ou_blocked".into(),
                    content: "blocked".into(),
                    channel: "feishu".into(),
                    timestamp: 0,
                    chat_type: Some("p2p".into()),
                    mentioned_open_ids: vec![],
                })
                .await
                .ok();
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                Ok(())
            }
        }

        let events: Arc<dyn EventListenerPort> = Arc::new(FilterTestEvents);
        let kernel = FeishuKernel::new(EventBus::default());
        let mut config = test_config();
        config.allowed_users = vec!["ou_allowed".into()];

        let svc = FeishuChannelService::new(
            config, auth, sender, bot_info, events, security, kernel,
        );

        let (tx, mut rx) = tokio::sync::mpsc::channel(10);
        svc.listen(tx).await.unwrap();

        let msg = rx.try_recv().unwrap();
        assert_eq!(msg.sender, "ou_allowed");
        assert!(rx.try_recv().is_err());
    }
}
