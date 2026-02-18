//! Event adapter -- wraps open-lark's WebSocket client.
//!
//! Uses `LarkWsClient::open()` to connect to Feishu's long-poll WebSocket.
//! open-lark's `EventDispatcherHandler` is not `Send`; we spawn the WS client
//! on a dedicated OS thread with its own tokio runtime.

use std::sync::Arc;

use async_trait::async_trait;
use feishu_domain::codec::decode_message_content;
use feishu_domain::{ChannelMessage, EventListenerPort};
use feishu_kernel::event_bus::{EventBus, FeishuEvent};
use open_lark::client::LarkClient;
use tracing::{error, info, warn};

const MAX_RECONNECT_ATTEMPTS: u32 = 10;
const RECONNECT_BASE_DELAY_SECS: u64 = 2;

/// Adapter wrapping open-lark's WebSocket event client.
pub struct LarkWsAdapter {
    client: Arc<LarkClient>,
    event_bus: Option<EventBus>,
}

impl LarkWsAdapter {
    pub fn new(client: Arc<LarkClient>) -> Self {
        Self {
            client,
            event_bus: None,
        }
    }

    pub fn with_event_bus(mut self, bus: EventBus) -> Self {
        self.event_bus = Some(bus);
        self
    }
}

#[async_trait]
impl EventListenerPort for LarkWsAdapter {
    async fn listen(
        &self,
        tx: tokio::sync::mpsc::Sender<ChannelMessage>,
    ) -> anyhow::Result<()> {
        let config = self.client.config.clone();
        let event_bus = self.event_bus.clone();

        if let Some(ref bus) = event_bus {
            bus.publish(FeishuEvent::ConnectionStateChanged { connected: false });
        }

        let mut attempt: u32 = 0;

        loop {
            attempt += 1;
            if attempt > MAX_RECONNECT_ATTEMPTS {
                error!(
                    "WebSocket: exceeded max reconnect attempts ({})",
                    MAX_RECONNECT_ATTEMPTS
                );
                return Err(anyhow::anyhow!(
                    "WebSocket: exceeded max reconnect attempts"
                ));
            }

            if attempt > 1 {
                let delay = RECONNECT_BASE_DELAY_SECS * 2u64.pow((attempt - 2).min(5));
                warn!(
                    "WebSocket: reconnect attempt {}/{} in {}s",
                    attempt, MAX_RECONNECT_ATTEMPTS, delay
                );
                tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
            }

            let tx_clone = tx.clone();
            let config_clone = config.clone();
            let event_bus_clone = event_bus.clone();

            let handle = std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create tokio runtime for WS listener");

                rt.block_on(async move {
                    use open_lark::client::ws_client::LarkWsClient;
                    use open_lark::event::dispatcher::EventDispatcherHandler;

                    let tx = tx_clone;
                    let event_handler = EventDispatcherHandler::builder()
                        .register_p2_im_message_receive_v1(move |event| {
                            let msg = &event.event;
                            let sender_open_id = msg.sender.sender_id.open_id.clone();
                            let raw_content = msg.message.content.clone();
                            let message_id = msg.message.message_id.clone();
                            let message_type = msg.message.message_type.clone();
                            let chat_type = msg.message.chat_type.clone();
                            let timestamp: u64 =
                                msg.message.create_time.parse().unwrap_or(0);

                            let content = decode_message_content(&raw_content, &message_type);

                            let mentioned_open_ids: Vec<String> = msg
                                .message
                                .mentions
                                .as_ref()
                                .map(|ms| {
                                    ms.iter()
                                        .map(|m| m.id.open_id.clone())
                                        .collect()
                                })
                                .unwrap_or_default();

                            let channel_msg = ChannelMessage {
                                id: message_id,
                                sender: sender_open_id,
                                content,
                                channel: msg.message.chat_id.clone(),
                                timestamp,
                                chat_type: Some(chat_type),
                                mentioned_open_ids,
                            };

                            if let Err(e) = tx.try_send(channel_msg) {
                                error!("Failed to forward message to channel bus: {e}");
                            }
                        })
                        .expect("Failed to register event handler")
                        .build();

                    info!("WebSocket: connecting to Feishu event stream");
                    let cfg = Arc::new(config_clone);

                    if let Some(ref bus) = event_bus_clone {
                        bus.publish(FeishuEvent::ConnectionStateChanged {
                            connected: true,
                        });
                    }

                    let result = LarkWsClient::open(cfg, event_handler).await;

                    if let Some(ref bus) = event_bus_clone {
                        bus.publish(FeishuEvent::ConnectionStateChanged {
                            connected: false,
                        });
                    }

                    result.map_err(|e| anyhow::anyhow!("WebSocket connection failed: {e}"))
                })
            });

            let result = tokio::task::spawn_blocking(move || {
                handle
                    .join()
                    .map_err(|_| anyhow::anyhow!("WS listener thread panicked"))
            })
            .await
            .map_err(|e| anyhow::anyhow!("Join error: {e}"))??;

            match result {
                Ok(()) => {
                    info!("WebSocket: connection closed normally");
                    return Ok(());
                }
                Err(e) => {
                    warn!("WebSocket: connection error: {e}");
                }
            }
        }
    }
}
