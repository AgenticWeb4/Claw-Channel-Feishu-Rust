//! Composition root -- wires all crates together.
//!
//! This is the single entry point for creating a fully configured
//! Feishu channel. It:
//! 1. Creates the `LarkClient` (open-lark SDK)
//! 2. Wires all port adapters from feishu-platform
//! 3. Creates capability wrappers and registers them with the Kernel
//! 4. Returns a ready-to-use `FeishuChannelService`

use std::sync::Arc;

use feishu_domain::{
    AuthPort, BotInfoPort, EventListenerPort, FeishuConfig, MessageSenderPort,
    SecurityGuard,
};
use feishu_platform::{LarkAuthAdapter, LarkBotAdapter, LarkImAdapter, LarkWsAdapter};
use feishu_kernel::{EventBus, FeishuKernel};
use open_lark::client::LarkClient;
use open_lark::core::constants::AppType;

use crate::capability::{AuthCapability, BotCapability, ImCapability};
use crate::channel::FeishuChannelService;

/// Create a fully wired Feishu channel with Kernel + EventBus.
pub fn create_channel(config: FeishuConfig) -> FeishuChannelService {
    let base_url = config.domain.base_url().to_string();

    let client = Arc::new(
        LarkClient::builder(&config.app_id, &config.app_secret)
            .with_app_type(AppType::SelfBuild)
            .with_open_base_url(base_url)
            .with_enable_token_cache(true)
            .build(),
    );

    // Port adapters from feishu-platform
    let auth: Arc<dyn AuthPort> = Arc::new(LarkAuthAdapter::new(client.clone()));
    let sender: Arc<dyn MessageSenderPort> = Arc::new(LarkImAdapter::new(client.clone()));
    let bot_info: Arc<dyn BotInfoPort> =
        Arc::new(LarkBotAdapter::new(client.clone(), auth.clone()));

    let event_bus = EventBus::default();

    let events: Arc<dyn EventListenerPort> = Arc::new(
        LarkWsAdapter::new(client.clone()).with_event_bus(event_bus.clone()),
    );

    let dm_list = config.dm_allowlist().to_vec();
    let security = SecurityGuard::with_policies(
        dm_list,
        Some(config.effective_dm_policy()),
        config.group_allow_from.clone(),
        Some(config.effective_group_policy()),
    );

    // Kernel: register capabilities for lifecycle management
    let mut kernel = FeishuKernel::new(event_bus);
    kernel.register(Arc::new(AuthCapability(auth.clone())));
    kernel.register(Arc::new(BotCapability(bot_info.clone())));
    kernel.register(Arc::new(ImCapability(sender.clone())));

    FeishuChannelService::new(config, auth, sender, bot_info, events, security, kernel)
}
