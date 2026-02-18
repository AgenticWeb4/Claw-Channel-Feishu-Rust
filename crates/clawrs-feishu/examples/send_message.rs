//! Send a Feishu message using the Claw 4-crate hexagonal architecture.
//!
//! Usage:
//!   cargo run --example send_message -- <chat_id> "Your message"
//!
//! Required environment variables:
//!   FEISHU_APP_ID     - Feishu app ID (from Feishu Open Platform)
//!   FEISHU_APP_SECRET - Feishu app secret
//!   FEISHU_CHAT_ID    - Default chat ID (optional, can be passed as CLI arg)

use clawrs_feishu::{
    create_channel, Channel, FeishuConfig, FeishuConnectionMode, FeishuDomain,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app_id = std::env::var("FEISHU_APP_ID")
        .expect("FEISHU_APP_ID env var required. See .env.example");
    let app_secret = std::env::var("FEISHU_APP_SECRET")
        .expect("FEISHU_APP_SECRET env var required. See .env.example");

    let args: Vec<String> = std::env::args().collect();
    let default_chat_id = std::env::var("FEISHU_CHAT_ID").unwrap_or_default();
    let chat_id = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or_else(|| {
            if default_chat_id.is_empty() {
                panic!("Chat ID required: pass as first CLI argument or set FEISHU_CHAT_ID env var");
            }
            default_chat_id.as_str()
        });
    let message = args
        .get(2)
        .cloned()
        .unwrap_or_else(|| {
            format!(
                "[Claw] Rust 4-crate hexagonal architecture message sent!\n\n\
                Architecture: 4-crate Microkernel + Hexagonal + DDD\n\
                SDK: open-lark v0.14\n\
                Crates: feishu-domain -> feishu-kernel -> adapters -> clawrs-feishu\n\
                Time: {}",
                chrono_lite_now()
            )
        });

    let config = FeishuConfig {
        app_id,
        app_secret,
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
    };

    let channel = create_channel(config);

    println!("Claw Feishu Channel (4-crate Architecture)");
    println!("  feishu-domain -> feishu-kernel -> feishu-platform -> clawrs-feishu");
    println!();

    println!("Health check...");
    let healthy = channel.health_check().await;
    println!("  Healthy: {healthy}");

    println!("Sending message to {chat_id}...");
    channel.send(&message, chat_id).await?;
    println!("  Message sent successfully!");

    Ok(())
}

fn chrono_lite_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let hours = ((secs % 86400) / 3600 + 8) % 24;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02} CST")
}
