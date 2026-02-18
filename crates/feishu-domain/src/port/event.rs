//! Event capability driven port.

use async_trait::async_trait;
use crate::ChannelMessage;

/// Port for receiving inbound events via WebSocket or webhook.
///
/// The event listener connects to Feishu's long-poll WebSocket,
/// parses event envelopes, and forwards messages to the channel bus.
#[async_trait]
pub trait EventListenerPort: Send + Sync {
    /// Start listening for events and forward messages to the sender.
    async fn listen(
        &self,
        tx: tokio::sync::mpsc::Sender<ChannelMessage>,
    ) -> anyhow::Result<()>;
}
