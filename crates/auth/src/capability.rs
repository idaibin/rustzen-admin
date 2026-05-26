//! Shared capability constants used by auth and permission checks.

pub const SYSTEM_WILDCARD: &str = "*";

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
pub mod system_dict {
    pub const LIST: &str = "system:dict:list";
    pub const CREATE: &str = "system:dict:create";
    pub const UPDATE: &str = "system:dict:update";
    pub const DELETE: &str = "system:dict:delete";
    pub const OPTIONS: &str = "system:dict:options";
}

/// Log management capability boundaries.
pub mod system_log {
    pub const LIST: &str = "system:log:list";
    pub const EXPORT: &str = "system:log:export";
}
