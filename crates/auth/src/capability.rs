//! Shared capability constants used by auth and permission checks.

pub const SYSTEM_WILDCARD: &str = "*";
pub const BUILTIN_OWNER_ROLE_CODE: &str = "owner";
pub const BUILTIN_ADMIN_ROLE_CODE: &str = "admin";
pub const BUILTIN_VIEWER_ROLE_CODE: &str = "viewer";

const DEPLOY_CAPABILITY_PREFIX: &str = "manage:deploy:";
const DEPLOY_VIEW_CAPABILITY: &str = "manage:deploy:list";
const OWNER_ONLY_CAPABILITY_ROOTS: &[&str] =
    &["system:module", "system:status", "manage:task", "manage:deploy"];
const VIEW_ACTIONS: &[&str] = &["list", "view", "options"];

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct RolePolicy;

impl RolePolicy {
    pub fn role_allows_capability(self, role_code: &str, capability_code: &str) -> bool {
        match role_code {
            BUILTIN_OWNER_ROLE_CODE => capability_code == SYSTEM_WILDCARD,
            BUILTIN_ADMIN_ROLE_CODE => self.is_assignable_leaf_capability(capability_code),
            BUILTIN_VIEWER_ROLE_CODE => {
                self.is_assignable_leaf_capability(capability_code)
                    && self.is_view_capability(capability_code)
            }
            _ => false,
        }
    }

    pub fn is_assignable_leaf_capability(self, capability_code: &str) -> bool {
        capability_code != SYSTEM_WILDCARD
            && !capability_code.ends_with(":*")
            && !self.is_owner_only_capability(capability_code)
    }

    pub fn is_view_capability(self, capability_code: &str) -> bool {
        capability_code.rsplit(':').next().is_some_and(|action| VIEW_ACTIONS.contains(&action))
    }

    pub fn is_deploy_capability(self, capability_code: &str) -> bool {
        capability_code == format!("{DEPLOY_CAPABILITY_PREFIX}*")
            || capability_code.starts_with(DEPLOY_CAPABILITY_PREFIX)
    }

    pub fn is_deploy_operation_capability(self, capability_code: &str) -> bool {
        self.is_deploy_capability(capability_code) && capability_code != DEPLOY_VIEW_CAPABILITY
    }

    pub fn is_owner_only_capability(self, capability_code: &str) -> bool {
        OWNER_ONLY_CAPABILITY_ROOTS.iter().any(|root| {
            capability_code == *root || capability_code.starts_with(&format!("{root}:"))
        })
    }

    pub fn is_owner_only_capability_or_wildcard(self, capability_code: &str) -> bool {
        if self.is_owner_only_capability(capability_code) {
            return true;
        }
        let Some(wildcard_prefix) = capability_code.strip_suffix('*') else {
            return false;
        };
        OWNER_ONLY_CAPABILITY_ROOTS
            .iter()
            .any(|root| format!("{root}:").starts_with(wildcard_prefix))
    }
}

pub fn is_deploy_capability_code(code: &str) -> bool {
    RolePolicy.is_deploy_capability(code)
}

#[cfg(test)]
mod role_policy_tests {
    use super::{
        BUILTIN_ADMIN_ROLE_CODE, BUILTIN_OWNER_ROLE_CODE, BUILTIN_VIEWER_ROLE_CODE, RolePolicy,
        SYSTEM_WILDCARD,
    };

    #[test]
    fn built_in_roles_keep_expected_capability_boundaries() {
        let policy = RolePolicy;
        assert!(policy.role_allows_capability(BUILTIN_OWNER_ROLE_CODE, SYSTEM_WILDCARD));
        assert!(policy.role_allows_capability(BUILTIN_ADMIN_ROLE_CODE, "system:user:create"));
        for capability in [
            "system:module",
            "system:module:list",
            "system:status",
            "system:status:view",
            "manage:task",
            "manage:task:list",
            "manage:deploy",
            "manage:deploy:list",
        ] {
            assert!(!policy.role_allows_capability(BUILTIN_ADMIN_ROLE_CODE, capability));
            assert!(!policy.role_allows_capability(BUILTIN_VIEWER_ROLE_CODE, capability));
            assert!(policy.is_owner_only_capability_or_wildcard(capability));
        }
        assert!(policy.is_owner_only_capability_or_wildcard("system:*"));
        assert!(policy.is_owner_only_capability_or_wildcard("manage:*"));
        assert!(!policy.is_owner_only_capability_or_wildcard("system:user:*"));
        assert!(policy.role_allows_capability(BUILTIN_VIEWER_ROLE_CODE, "system:user:list"));
        assert!(!policy.role_allows_capability(BUILTIN_VIEWER_ROLE_CODE, "system:user:create"));
        for module in ["monitor", "insights", "reports"] {
            assert!(
                policy.role_allows_capability(BUILTIN_ADMIN_ROLE_CODE, &format!("{module}:view"))
            );
            assert!(
                policy.role_allows_capability(BUILTIN_ADMIN_ROLE_CODE, &format!("{module}:manage"))
            );
            assert!(
                policy.role_allows_capability(BUILTIN_VIEWER_ROLE_CODE, &format!("{module}:view"))
            );
            assert!(
                !policy
                    .role_allows_capability(BUILTIN_VIEWER_ROLE_CODE, &format!("{module}:manage"))
            );
        }
        for capability in [
            "monitor:overview:view",
            "monitor:node:view",
            "monitor:incident:view",
            "insights:overview:view",
            "insights:event:view",
            "reports:run:view",
            "reports:flow:view",
            "reports:schedule:view",
        ] {
            assert!(policy.role_allows_capability(BUILTIN_ADMIN_ROLE_CODE, capability));
            assert!(policy.role_allows_capability(BUILTIN_VIEWER_ROLE_CODE, capability));
        }
    }
}

/// Dashboard capability boundary.
pub mod dashboard {
    pub const VIEW: &str = "dashboard:view";
}

/// Monitor capability boundaries.
pub mod monitor {
    pub const OVERVIEW_VIEW: &str = "monitor:overview:view";
    pub const NODE_VIEW: &str = "monitor:node:view";
    pub const INCIDENT_VIEW: &str = "monitor:incident:view";
    pub const MANAGE: &str = "monitor:manage";
}

/// Insights capability boundaries.
pub mod insights {
    pub const OVERVIEW_VIEW: &str = "insights:overview:view";
    pub const EVENT_VIEW: &str = "insights:event:view";
    pub const MANAGE: &str = "insights:manage";
}

/// Reports capability boundaries.
pub mod reports {
    pub const RUN_VIEW: &str = "reports:run:view";
    pub const RUN_MANAGE: &str = "reports:run:manage";
    pub const SYSTEM_VIEW: &str = "reports:system:view";
    pub const SYSTEM_MANAGE: &str = "reports:system:manage";
    pub const FLOW_VIEW: &str = "reports:flow:view";
    pub const FLOW_MANAGE: &str = "reports:flow:manage";
    pub const SCHEDULE_VIEW: &str = "reports:schedule:view";
    pub const SCHEDULE_MANAGE: &str = "reports:schedule:manage";
    pub const MANAGE: &str = "reports:manage";
}

/// User management capability boundaries.
pub mod system_user {
    pub const LIST: &str = "system:user:list";
    pub const CREATE: &str = "system:user:create";
    pub const UPDATE: &str = "system:user:update";
    pub const DELETE: &str = "system:user:delete";
    pub const OPTIONS: &str = "system:user:options";
    pub const RESET_PASSWORD: &str = "system:user:password";
    pub const UPDATE_STATUS: &str = "system:user:status";
}

/// Role management capability boundaries.
pub mod system_role {
    pub const LIST: &str = "system:role:list";
    pub const CREATE: &str = "system:role:create";
    pub const UPDATE: &str = "system:role:update";
    pub const DELETE: &str = "system:role:delete";
    pub const OPTIONS: &str = "system:role:options";
}

/// Menu management capability boundaries.
pub mod system_menu {
    pub const LIST: &str = "system:menu:list";
    pub const UPDATE: &str = "system:menu:update";
    pub const DELETE: &str = "system:menu:delete";
    pub const OPTIONS: &str = "system:menu:options";
}

/// System status capability boundary.
pub mod system_status {
    pub const VIEW: &str = "system:status:view";
}

/// Owner-only module process-log diagnostics capability boundaries.
pub mod system_module_log {
    pub const VIEW: &str = "system:module:log:view";
    pub const BACKUP: &str = "system:module:log:backup";
    pub const CLEANUP: &str = "system:module:log:cleanup";
}

/// Independent module administration capability boundaries.
pub mod system_module {
    pub const LIST: &str = "system:module:list";
    pub const UPDATE: &str = "system:module:update";
}

/// Log management capability boundaries.
pub mod manage_log {
    pub const LIST: &str = "manage:log:list";
    pub const EXPORT: &str = "manage:log:export";
}

/// Scheduled task capability boundaries.
pub mod manage_task {
    pub const LIST: &str = "manage:task:list";
    pub const RUN: &str = "manage:task:run";
}

/// Deploy version capability boundaries.
pub mod manage_deploy {
    pub const LIST: &str = "manage:deploy:list";
    pub const CREATE: &str = "manage:deploy:create";
    pub const UPDATE: &str = "manage:deploy:update";
    pub const DELETE: &str = "manage:deploy:delete";
    pub const RUN: &str = "manage:deploy:run";
}
