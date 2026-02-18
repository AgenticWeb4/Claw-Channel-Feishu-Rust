//! Security policies for DM and group message handling.
//!
//! Mirrors OpenClaw's `dmPolicy` / `groupPolicy` configuration.

use serde::{Deserialize, Serialize};

/// Policy for handling direct messages.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DmPolicy {
    /// Accept DMs from paired (allow-listed) users only.
    #[default]
    Pairing,
    /// Accept DMs from any user.
    Open,
    /// Reject all DMs.
    Deny,
}

/// Policy for handling group messages.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GroupPolicy {
    /// Accept from allow-listed groups only.
    #[default]
    Allowlist,
    /// Accept from any group the bot is a member of.
    Open,
    /// Reject all group messages.
    Deny,
}

impl DmPolicy {
    /// Derive policy from allowlist: empty = Deny, ["*"] = Open, else = Pairing.
    pub fn from_allowlist(list: &[String]) -> Self {
        if list.is_empty() {
            Self::Deny
        } else if list.iter().any(|u| u == "*") {
            Self::Open
        } else {
            Self::Pairing
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policies() {
        assert_eq!(DmPolicy::default(), DmPolicy::Pairing);
        assert_eq!(GroupPolicy::default(), GroupPolicy::Allowlist);
    }

    #[test]
    fn dm_policy_from_allowlist() {
        assert_eq!(DmPolicy::from_allowlist(&[]), DmPolicy::Deny);
        assert_eq!(
            DmPolicy::from_allowlist(&["*".into()]),
            DmPolicy::Open
        );
        assert_eq!(
            DmPolicy::from_allowlist(&["ou_abc".into()]),
            DmPolicy::Pairing
        );
    }
}
