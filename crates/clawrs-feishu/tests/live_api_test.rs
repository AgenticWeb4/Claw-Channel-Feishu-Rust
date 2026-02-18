//! Live API verification -- tests against real Feishu API.
//!
//! These tests require real Feishu credentials and are gated behind `#[ignore]`.
//! They do NOT run with `cargo test` by default.
//!
//! Run explicitly:
//!   FEISHU_APP_ID=xxx FEISHU_APP_SECRET=yyy cargo test -p clawrs-feishu --test live_api_test -- --ignored --nocapture

use std::sync::Arc;

use open_lark::client::LarkClient;
use open_lark::core::constants::AppType;
use clawrs_feishu::{
    create_channel, AuthPort, BotInfoPort, Channel, FeishuConfig,
    FeishuConnectionMode, FeishuDomain,
};
use feishu_platform::{LarkAuthAdapter, LarkBotAdapter};

fn env_or_skip(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| {
        panic!(
            "Env var {key} not set. Live tests require real Feishu credentials.\n\
             Set FEISHU_APP_ID and FEISHU_APP_SECRET before running."
        )
    })
}

fn live_config() -> FeishuConfig {
    FeishuConfig {
        app_id: env_or_skip("FEISHU_APP_ID"),
        app_secret: env_or_skip("FEISHU_APP_SECRET"),
        domain: FeishuDomain::Feishu,
        connection_mode: FeishuConnectionMode::WebSocket,
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

fn make_lark_client() -> Arc<LarkClient> {
    let app_id = env_or_skip("FEISHU_APP_ID");
    let app_secret = env_or_skip("FEISHU_APP_SECRET");
    Arc::new(
        LarkClient::builder(&app_id, &app_secret)
            .with_app_type(AppType::SelfBuild)
            .with_enable_token_cache(true)
            .build(),
    )
}

#[tokio::test]
#[ignore] // Requires real Feishu credentials
async fn live_auth_token_fetch() {
    let client = make_lark_client();
    let auth = LarkAuthAdapter::new(client);

    match auth.get_token().await {
        Ok(token) => {
            println!("Auth token obtained: {}...", &token[..token.len().min(20)]);
            assert!(!token.is_empty(), "Token should not be empty");
        }
        Err(e) => {
            panic!("Auth token fetch failed: {e}");
        }
    }
}

#[tokio::test]
#[ignore] // Requires real Feishu credentials
async fn live_bot_info() {
    let client = make_lark_client();
    let auth: Arc<dyn AuthPort> = Arc::new(LarkAuthAdapter::new(client.clone()));
    let bot = LarkBotAdapter::new(client, auth);

    match bot.get_bot_open_id().await {
        Ok(open_id) => {
            println!("Bot open_id: {open_id:?}");
            assert!(open_id.is_some(), "Bot should have an open_id");
        }
        Err(e) => {
            panic!("Bot info fetch failed: {e}");
        }
    }
}

#[tokio::test]
#[ignore] // Requires real Feishu credentials
async fn live_health_check() {
    let ch = create_channel(live_config());
    let healthy = ch.health_check().await;
    println!("Feishu health_check result: {healthy}");
    assert!(healthy, "health_check should succeed with real credentials");
}

#[tokio::test]
#[ignore] // Requires real Feishu credentials
async fn live_channel_name() {
    let ch = create_channel(live_config());
    assert_eq!(ch.name(), "feishu");
}
