//! Message filter logic for listen().
//!
//! Applies SecurityGuard policies and @mention detection before forwarding
//! messages to the application layer.

use feishu_domain::{ChannelMessage, SecurityGuard};

/// Parameters for the message filter.
#[derive(Clone)]
pub(super) struct FilterParams {
    pub security: SecurityGuard,
    pub bot_open_id: Option<String>,
    pub group_require_mention: bool,
}

/// Returns true if the message should be forwarded to the application.
pub(super) fn should_forward_message(msg: &ChannelMessage, params: &FilterParams) -> bool {
    let is_group = msg.chat_type.as_deref() == Some("group");

    if is_group {
        if !params.security.is_group_allowed(&msg.sender) {
            tracing::warn!(
                "Feishu: ignoring group message from unauthorized user: {}",
                msg.sender
            );
            return false;
        }
        if params.group_require_mention {
            if let Some(ref bot_id) = params.bot_open_id {
                let bot_mentioned = msg.mentioned_open_ids.iter().any(|id| id == bot_id);
                if !bot_mentioned {
                    tracing::debug!(
                        "Feishu: ignoring group message (bot not mentioned)"
                    );
                    return false;
                }
            }
        }
    } else {
        if !params.security.is_dm_allowed(&msg.sender) {
            tracing::warn!(
                "Feishu: ignoring DM from unauthorized user: {}",
                msg.sender
            );
            return false;
        }
    }

    true
}
