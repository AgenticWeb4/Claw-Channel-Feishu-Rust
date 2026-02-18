//! Message codec -- domain service for encoding/decoding Feishu message content.
//!
//! Pure functions with no I/O dependencies.

use crate::model::event::FeishuMention;

/// Decode Feishu message content JSON into plain text.
pub fn decode_message_content(content: &str, message_type: &str) -> String {
    match message_type {
        "text" => {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(content) {
                parsed
                    .get("text")
                    .and_then(|t| t.as_str())
                    .unwrap_or(content)
                    .to_string()
            } else {
                content.to_string()
            }
        }
        _ => content.to_string(),
    }
}

/// Check if the bot is mentioned in the message.
pub fn is_bot_mentioned(mentions: &[FeishuMention], bot_open_id: &str) -> bool {
    mentions
        .iter()
        .any(|m| m.id.open_id.as_deref() == Some(bot_open_id))
}

/// Strip bot @mention placeholders from message text.
pub fn strip_bot_mentions(text: &str, mentions: &[FeishuMention]) -> String {
    let mut result = text.to_string();
    for mention in mentions {
        result = result.replace(&mention.key, "");
        let at_name = format!("@{}", mention.name);
        result = result.replace(&at_name, "");
    }
    result.trim().to_string()
}

/// Encode a plain text string into Feishu text message JSON.
pub fn encode_text_message(text: &str) -> String {
    serde_json::json!({ "text": text }).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::event::FeishuSenderId;

    #[test]
    fn decode_text_message() {
        let content = r#"{"text":"Hello world"}"#;
        assert_eq!(decode_message_content(content, "text"), "Hello world");
    }

    #[test]
    fn decode_text_with_mention() {
        let content = r#"{"text":"@_user_1 Hello"}"#;
        assert_eq!(
            decode_message_content(content, "text"),
            "@_user_1 Hello"
        );
    }

    #[test]
    fn decode_unknown_type_returns_raw() {
        let content = r#"{"some":"data"}"#;
        assert_eq!(
            decode_message_content(content, "unknown"),
            r#"{"some":"data"}"#
        );
    }

    #[test]
    fn decode_invalid_json_returns_raw() {
        let content = "not json";
        assert_eq!(decode_message_content(content, "text"), "not json");
    }

    #[test]
    fn strip_bot_mention_removes_at_prefix() {
        let mentions = vec![FeishuMention {
            key: "@_user_1".into(),
            id: FeishuSenderId {
                open_id: Some("ou_bot".into()),
                user_id: None,
                union_id: None,
            },
            name: "MyBot".into(),
        }];
        let text = "@_user_1 Hello bot";
        assert_eq!(strip_bot_mentions(text, &mentions), "Hello bot");
    }

    #[test]
    fn strip_bot_mention_no_mentions() {
        assert_eq!(strip_bot_mentions("Hello", &[]), "Hello");
    }

    #[test]
    fn check_bot_mentioned_true() {
        let mentions = vec![FeishuMention {
            key: "@_user_1".into(),
            id: FeishuSenderId {
                open_id: Some("ou_bot_id".into()),
                user_id: None,
                union_id: None,
            },
            name: "Bot".into(),
        }];
        assert!(is_bot_mentioned(&mentions, "ou_bot_id"));
    }

    #[test]
    fn check_bot_mentioned_false() {
        let mentions = vec![FeishuMention {
            key: "@_user_1".into(),
            id: FeishuSenderId {
                open_id: Some("ou_other".into()),
                user_id: None,
                union_id: None,
            },
            name: "Other".into(),
        }];
        assert!(!is_bot_mentioned(&mentions, "ou_bot_id"));
    }

    #[test]
    fn encode_text_message_json() {
        let json = encode_text_message("Hello");
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["text"], "Hello");
    }
}
