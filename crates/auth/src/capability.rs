//! Shared capability constants used by auth and permission checks.

pub const SYSTEM_WILDCARD: &str = "*";

pub fn is_deploy_capability_code(code: &str) -> bool {
    code == "manage:deploy:*" || code.starts_with("manage:deploy:")
}

/// Dashboard capability boundary.
pub mod dashboard {
    pub const VIEW: &str = "dashboard:view";
}

/// User management capability boundaries.
pub mod system_user {
    pub const LIST: &str = "system:user:list";
    pub const CREATE: &str = "system:user:create";
    pub const UPDATE: &str = "system:user:update";
    pub const DELETE: &str = "system:user:delete";
    pub const OPTIONS: &str = "system:user:list";
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
    pub const CREATE: &str = "system:menu:create";
    pub const UPDATE: &str = "system:menu:update";
    pub const DELETE: &str = "system:menu:delete";
    pub const OPTIONS: &str = "system:menu:options";
}

/// Dictionary management capability boundaries.
pub mod manage_dict {
    pub const LIST: &str = "manage:dict:list";
    pub const CREATE: &str = "manage:dict:create";
    pub const UPDATE: &str = "manage:dict:update";
    pub const DELETE: &str = "manage:dict:delete";
    pub const OPTIONS: &str = "manage:dict:options";
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
