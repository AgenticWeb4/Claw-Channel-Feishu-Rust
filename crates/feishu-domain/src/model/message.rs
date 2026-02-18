/// A message received from or sent to a channel.
///
/// Value object mirroring `zeroclaw_core::channels::traits::ChannelMessage`.
/// Defined locally so the feishu crates compile independently.
#[derive(Debug, Clone)]
pub struct ChannelMessage {
    pub id: String,
    pub sender: String,
    pub content: String,
    pub channel: String,
    pub timestamp: u64,
    /// "p2p" or "group" -- needed for @mention filtering at application layer.
    pub chat_type: Option<String>,
    /// Open IDs of users mentioned in this message -- needed for bot-mention detection.
    pub mentioned_open_ids: Vec<String>,
}
