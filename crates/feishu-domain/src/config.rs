use serde::{Deserialize, Serialize};

use crate::security::{DmPolicy, GroupPolicy};

/// Feishu API domain -- China (feishu.cn) or International (larksuite.com)
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FeishuDomain {
    #[default]
    Feishu,
    Lark,
}

impl FeishuDomain {
    pub fn base_url(&self) -> &str {
        match self {
            Self::Feishu => "https://open.feishu.cn",
            Self::Lark => "https://open.larksuite.com",
        }
    }
}

/// Connection mode for receiving Feishu events
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FeishuConnectionMode {
    #[default]
    WebSocket,
    Webhook,
}

/// Feishu channel configuration (from config.toml [channels_config.feishu])
///
/// OpenClaw-compatible: dm_policy, group_policy, allow_from, group_allow_from.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuConfig {
    /// Feishu App ID (from open platform)
    pub app_id: String,
    /// Feishu App Secret
    pub app_secret: String,
    /// API domain: feishu (China) or lark (International)
    #[serde(default)]
    pub domain: FeishuDomain,
    /// Connection mode: websocket (default) or webhook
    #[serde(default)]
    pub connection_mode: FeishuConnectionMode,
    /// Allowed user open_ids for DM. Empty = deny all, ["*"] = allow all.
    /// Alias: allow_from (OpenClaw compat). Prefer allow_from if both set.
    #[serde(default)]
    pub allowed_users: Vec<String>,
    /// DM policy: pairing/allowlist, open, deny. Default derived from allowed_users.
    #[serde(default)]
    pub dm_policy: Option<DmPolicy>,
    /// Group policy: allowlist, open, deny. Default: open.
    #[serde(default)]
    pub group_policy: Option<GroupPolicy>,
    /// DM allowlist (OpenClaw allowFrom). Overrides allowed_users when set.
    #[serde(default)]
    pub allow_from: Option<Vec<String>>,
    /// Group allowlist (OpenClaw groupAllowFrom). Used when group_policy = allowlist.
    #[serde(default)]
    pub group_allow_from: Vec<String>,
    /// Require @mention in groups. Default: true.
    #[serde(default = "default_true")]
    pub group_require_mention: bool,
    /// Event encrypt key (for webhook mode decryption)
    pub encrypt_key: Option<String>,
    /// Event verification token
    pub verification_token: Option<String>,
    /// Webhook server port (webhook mode only)
    pub webhook_port: Option<u16>,
}

fn default_true() -> bool {
    true
}

impl FeishuConfig {
    /// Effective DM allowlist: allow_from or allowed_users.
    pub fn dm_allowlist(&self) -> &[String] {
        self.allow_from
            .as_deref()
            .unwrap_or(&self.allowed_users)
    }

    /// Effective DM policy. When None, derive: empty=Deny, ["*"]=Open, else=Pairing.
    pub fn effective_dm_policy(&self) -> DmPolicy {
        self.dm_policy
            .unwrap_or_else(|| DmPolicy::from_allowlist(self.dm_allowlist()))
    }

    /// Effective group policy. Default: Open.
    pub fn effective_group_policy(&self) -> GroupPolicy {
        self.group_policy.unwrap_or(GroupPolicy::Open)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feishu_domain_default_is_feishu() {
        let domain = FeishuDomain::default();
        assert_eq!(domain.base_url(), "https://open.feishu.cn");
    }

    #[test]
    fn feishu_domain_lark() {
        let domain = FeishuDomain::Lark;
        assert_eq!(domain.base_url(), "https://open.larksuite.com");
    }

    #[test]
    fn feishu_connection_mode_default_is_websocket() {
        let mode = FeishuConnectionMode::default();
        assert!(matches!(mode, FeishuConnectionMode::WebSocket));
    }

    #[test]
    fn feishu_config_deserialize_minimal() {
        let toml_str = r#"
            app_id = "cli_test"
            app_secret = "secret123"
        "#;
        let config: FeishuConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.app_id, "cli_test");
        assert_eq!(config.app_secret, "secret123");
        assert!(matches!(config.domain, FeishuDomain::Feishu));
        assert!(matches!(
            config.connection_mode,
            FeishuConnectionMode::WebSocket
        ));
        assert!(config.allowed_users.is_empty());
    }

    #[test]
    fn feishu_config_deserialize_full() {
        let toml_str = r#"
            app_id = "cli_full"
            app_secret = "secret_full"
            domain = "lark"
            connection_mode = "webhook"
            allowed_users = ["ou_abc", "ou_def"]
            encrypt_key = "enc123"
            verification_token = "vtoken"
            webhook_port = 8081
        "#;
        let config: FeishuConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.app_id, "cli_full");
        assert!(matches!(config.domain, FeishuDomain::Lark));
        assert!(matches!(
            config.connection_mode,
            FeishuConnectionMode::Webhook
        ));
        assert_eq!(config.allowed_users, vec!["ou_abc", "ou_def"]);
        assert_eq!(config.encrypt_key.unwrap(), "enc123");
        assert_eq!(config.webhook_port.unwrap(), 8081);
    }

    #[test]
    fn feishu_config_empty_allowlist() {
        let toml_str = r#"
            app_id = "cli_test"
            app_secret = "secret"
        "#;
        let config: FeishuConfig = toml::from_str(toml_str).unwrap();
        assert!(config.allowed_users.is_empty());
    }

    #[test]
    fn feishu_config_dm_group_policy() {
        let toml_str = r#"
            app_id = "cli_policy"
            app_secret = "secret"
            dm_policy = "open"
            group_policy = "allowlist"
            group_allow_from = ["ou_grp1"]
        "#;
        let config: FeishuConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.effective_dm_policy(), DmPolicy::Open);
        assert_eq!(config.effective_group_policy(), GroupPolicy::Allowlist);
        assert_eq!(config.group_allow_from, vec!["ou_grp1"]);
    }
}
