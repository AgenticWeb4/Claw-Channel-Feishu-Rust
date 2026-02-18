//! Inter-capability event bus.
//!
//! Lightweight broadcast channel for decoupled communication between
//! capabilities.  E.g. the event capability publishes `MessageReceived`
//! and the IM capability (or any future consumer) subscribes to it.

use feishu_domain::ChannelMessage;
use tokio::sync::broadcast;

/// Events that flow between capabilities via the kernel.
#[derive(Debug, Clone)]
pub enum FeishuEvent {
    /// A new inbound message was received (from WebSocket or webhook).
    MessageReceived(ChannelMessage),

    /// The tenant_access_token was refreshed.
    TokenRefreshed { expires_in_secs: u64 },

    /// WebSocket connection state changed.
    ConnectionStateChanged { connected: bool },
}

/// Broadcast-based event bus shared by all capabilities.
#[derive(Debug, Clone)]
pub struct EventBus {
    sender: broadcast::Sender<FeishuEvent>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(256)
    }
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Publish an event to all subscribers.
    pub fn publish(&self, event: FeishuEvent) {
        let _ = self.sender.send(event);
    }

    /// Subscribe to events.
    pub fn subscribe(&self) -> broadcast::Receiver<FeishuEvent> {
        self.sender.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn event_bus_publish_subscribe() {
        let bus = EventBus::default();
        let mut rx = bus.subscribe();

        bus.publish(FeishuEvent::ConnectionStateChanged { connected: true });

        match rx.recv().await.unwrap() {
            FeishuEvent::ConnectionStateChanged { connected } => assert!(connected),
            _ => panic!("unexpected event type"),
        }
    }

    #[tokio::test]
    async fn event_bus_multiple_subscribers() {
        let bus = EventBus::default();
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        bus.publish(FeishuEvent::TokenRefreshed {
            expires_in_secs: 7200,
        });

        match rx1.recv().await.unwrap() {
            FeishuEvent::TokenRefreshed { expires_in_secs } => {
                assert_eq!(expires_in_secs, 7200)
            }
            _ => panic!("unexpected event type"),
        }
        match rx2.recv().await.unwrap() {
            FeishuEvent::TokenRefreshed { expires_in_secs } => {
                assert_eq!(expires_in_secs, 7200)
            }
            _ => panic!("unexpected event type"),
        }
    }
}
