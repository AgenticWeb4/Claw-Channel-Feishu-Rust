//! Allowlist-based security guard (OpenClaw-compatible).
//!
//! Domain service enforcing access control:
//! - DM: dm_policy (Deny / Pairing=allowlist / Open)
//! - Group: group_policy (Deny / Allowlist / Open)

use crate::security::{DmPolicy, GroupPolicy};

/// Security guard with DM and group policies.
#[derive(Debug, Clone)]
pub struct SecurityGuard {
    dm_allowlist: Vec<String>,
    dm_policy: DmPolicy,
    group_allowlist: Vec<String>,
    group_policy: GroupPolicy,
}

impl SecurityGuard {
    /// Create from explicit policy and allowlists.
    pub fn new(allowed_users: Vec<String>) -> Self {
        Self::with_policies(
            allowed_users,
            None,
            vec![],
            None,
        )
    }

    /// Create with full OpenClaw-compatible policy.
    pub fn with_policies(
        dm_allowlist: Vec<String>,
        dm_policy: Option<DmPolicy>,
        group_allowlist: Vec<String>,
        group_policy: Option<GroupPolicy>,
    ) -> Self {
        let dm_policy = dm_policy.unwrap_or_else(|| DmPolicy::from_allowlist(&dm_allowlist));
        let group_policy = group_policy.unwrap_or(GroupPolicy::Open);

        Self {
            dm_allowlist,
            dm_policy,
            group_allowlist,
            group_policy,
        }
    }

    /// Check if a user is allowed for DM (legacy / backward compat).
    pub fn is_user_allowed(&self, user_id: &str) -> bool {
        self.is_dm_allowed(user_id)
    }

    /// Check if a user is allowed for DM per dm_policy.
    pub fn is_dm_allowed(&self, user_id: &str) -> bool {
        if user_id.is_empty() {
            return false;
        }
        match self.dm_policy {
            DmPolicy::Deny => false,
            DmPolicy::Open => true,
            DmPolicy::Pairing => self
                .dm_allowlist
                .iter()
                .any(|u| u == "*" || u == user_id),
        }
    }

    /// Check if a user is allowed for group per group_policy.
    pub fn is_group_allowed(&self, user_id: &str) -> bool {
        if user_id.is_empty() {
            return false;
        }
        match self.group_policy {
            GroupPolicy::Deny => false,
            GroupPolicy::Open => true,
            GroupPolicy::Allowlist => self
                .group_allowlist
                .iter()
                .any(|u| u == "*" || u == user_id),
        }
    }

    /// Check if any of the provided identities is allowed for DM.
    pub fn is_any_allowed(&self, identities: &[&str]) -> bool {
        identities.iter().any(|id| self.is_dm_allowed(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_allowlist_denies_everyone() {
        let guard = SecurityGuard::new(vec![]);
        assert!(!guard.is_user_allowed("ou_abc"));
        assert!(!guard.is_dm_allowed("ou_abc"));
    }

    #[test]
    fn wildcard_allows_everyone() {
        let guard = SecurityGuard::new(vec!["*".into()]);
        assert!(guard.is_user_allowed("ou_abc"));
        assert!(guard.is_dm_allowed("anyone"));
    }

    #[test]
    fn specific_allowlist() {
        let guard = SecurityGuard::new(vec!["ou_abc".into(), "ou_def".into()]);
        assert!(guard.is_dm_allowed("ou_abc"));
        assert!(guard.is_dm_allowed("ou_def"));
        assert!(!guard.is_dm_allowed("ou_xyz"));
    }

    #[test]
    fn exact_match_not_substring() {
        let guard = SecurityGuard::new(vec!["ou_abc".into()]);
        assert!(!guard.is_dm_allowed("ou_abcdef"));
        assert!(!guard.is_dm_allowed("ou_ab"));
    }

    #[test]
    fn empty_user_denied() {
        let guard = SecurityGuard::new(vec!["ou_abc".into()]);
        assert!(!guard.is_dm_allowed(""));
    }

    #[test]
    fn any_identity_allowed() {
        let guard = SecurityGuard::new(vec!["ou_abc".into()]);
        assert!(guard.is_any_allowed(&["unknown", "ou_abc"]));
        assert!(!guard.is_any_allowed(&["unknown", "ou_xyz"]));
    }

    #[test]
    fn dm_policy_deny() {
        let guard = SecurityGuard::with_policies(
            vec!["ou_abc".into()],
            Some(DmPolicy::Deny),
            vec![],
            None,
        );
        assert!(!guard.is_dm_allowed("ou_abc"));
    }

    #[test]
    fn dm_policy_open() {
        let guard = SecurityGuard::with_policies(
            vec![],
            Some(DmPolicy::Open),
            vec![],
            None,
        );
        assert!(guard.is_dm_allowed("ou_anyone"));
    }

    #[test]
    fn group_policy_open() {
        let guard = SecurityGuard::with_policies(
            vec![],
            Some(DmPolicy::Deny),
            vec![],
            Some(GroupPolicy::Open),
        );
        assert!(guard.is_group_allowed("ou_anyone"));
    }

    #[test]
    fn group_policy_allowlist() {
        let guard = SecurityGuard::with_policies(
            vec![],
            Some(DmPolicy::Deny),
            vec!["ou_grp1".into()],
            Some(GroupPolicy::Allowlist),
        );
        assert!(guard.is_group_allowed("ou_grp1"));
        assert!(!guard.is_group_allowed("ou_grp2"));
    }
}
