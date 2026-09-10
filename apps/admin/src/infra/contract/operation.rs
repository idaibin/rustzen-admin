use axum::http::Method;

#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "Public, Any, and All are validated by the contract harness before a production route needs them"
)]
pub enum AccessPolicy {
    Public,
    Authenticated,
    Require(&'static str),
    Any(Vec<&'static str>),
    All(Vec<&'static str>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RouteContract {
    pub method: Method,
    pub path: String,
    pub operation: OperationDescriptor,
    pub access: RegisteredAccess,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(dead_code, reason = "synthetic policy contracts are exercised by focused tests")]
pub enum OperationDescriptor {
    Login,
    CurrentAdminUser,
    Logout,
    UpdateAccountAvatar,
    UpdateAccountProfile,
    ChangeAccountPassword,
    GetDashboardStats,
    ListManageLogs,
    ExportManageLogs,
    ListManageTasks,
    ListTaskRuns,
    RunTask,
    ListDeployments,
    UploadDeployment,
    CleanupDeployments,
    GetDeployment,
    ExpireDeployment,
    DeleteDeployment,
    DeployVersion,
    ListMenus,
    ListModuleMenuInventory,
    UpdateMenu,
    DeleteMenu,
    GetMenuOptions,
    ListRoles,
    CreateRole,
    UpdateRole,
    DeleteRole,
    GetRoleOptions,
    GetStatusOverview,
    ListModuleLogs,
    TailModuleLog,
    BackupModuleLogs,
    PreviewModuleLogCleanup,
    ConfirmModuleLogCleanup,
    ListUsers,
    CreateAdminUser,
    UpdateUser,
    DeleteUser,
    GetUserOptions,
    GetUserStatusOptions,
    UpdateUserPassword,
    UpdateUserStatus,
    RevokeUserSessions,
    ListModules,
    UpdateModuleEnabled,
    GetModuleNavigation,
    GetDashboardModules,
    #[cfg(feature = "selected-distribution")]
    GetWebBinding,
    #[cfg(feature = "selected-distribution")]
    GetInstallation,
    ListNotifications,
    GetNotificationUnreadCount,
    GetNotification,
    ReadNotification,
    ReadAllNotifications,
    StreamNotifications,
    ContractPublic,
    ContractAny,
    ContractAll,
    BenchmarkRequire,
}
impl OperationDescriptor {
    pub const fn operation_id(&self) -> &'static str {
        match self {
            Self::Login => "login",
            Self::CurrentAdminUser => "getCurrentAdminUser",
            Self::Logout => "logout",
            Self::UpdateAccountAvatar => "updateAccountAvatar",
            Self::UpdateAccountProfile => "updateAccountProfile",
            Self::ChangeAccountPassword => "changeAccountPassword",
            Self::GetDashboardStats => "getDashboardStats",
            Self::ListManageLogs => "listManageLogs",
            Self::ExportManageLogs => "exportManageLogs",
            Self::ListManageTasks => "listManageTasks",
            Self::ListTaskRuns => "listTaskRuns",
            Self::RunTask => "runTask",
            Self::ListDeployments => "listDeployments",
            Self::UploadDeployment => "uploadDeployment",
            Self::CleanupDeployments => "cleanupDeployments",
            Self::GetDeployment => "getDeployment",
            Self::ExpireDeployment => "expireDeployment",
            Self::DeleteDeployment => "deleteDeployment",
            Self::DeployVersion => "deployVersion",
            Self::ListMenus => "listMenus",
            Self::ListModuleMenuInventory => "listModuleMenuInventory",
            Self::UpdateMenu => "updateMenu",
            Self::DeleteMenu => "deleteMenu",
            Self::GetMenuOptions => "getMenuOptions",
            Self::ListRoles => "listRoles",
            Self::CreateRole => "createRole",
            Self::UpdateRole => "updateRole",
            Self::DeleteRole => "deleteRole",
            Self::GetRoleOptions => "getRoleOptions",
            Self::GetStatusOverview => "getStatusOverview",
            Self::ListModuleLogs => "listModuleLogs",
            Self::TailModuleLog => "tailModuleLog",
            Self::BackupModuleLogs => "backupModuleLogs",
            Self::PreviewModuleLogCleanup => "previewModuleLogCleanup",
            Self::ConfirmModuleLogCleanup => "confirmModuleLogCleanup",
            Self::ListUsers => "listUsers",
            Self::CreateAdminUser => "createAdminUser",
            Self::UpdateUser => "updateUser",
            Self::DeleteUser => "deleteUser",
            Self::GetUserOptions => "getUserOptions",
            Self::GetUserStatusOptions => "getUserStatusOptions",
            Self::UpdateUserPassword => "updateUserPassword",
            Self::UpdateUserStatus => "updateUserStatus",
            Self::RevokeUserSessions => "revokeUserSessions",
            Self::ListModules => "listModules",
            Self::UpdateModuleEnabled => "updateModuleEnabled",
            Self::GetModuleNavigation => "getModuleNavigation",
            Self::GetDashboardModules => "getDashboardModules",
            #[cfg(feature = "selected-distribution")]
            Self::GetWebBinding => "getWebBinding",
            #[cfg(feature = "selected-distribution")]
            Self::GetInstallation => "getInstallation",
            Self::ListNotifications => "listNotifications",
            Self::GetNotificationUnreadCount => "getNotificationUnreadCount",
            Self::GetNotification => "getNotification",
            Self::ReadNotification => "readNotification",
            Self::ReadAllNotifications => "readAllNotifications",
            Self::StreamNotifications => "streamNotifications",
            Self::ContractPublic => "contractPublic",
            Self::ContractAny => "contractAny",
            Self::ContractAll => "contractAll",
            Self::BenchmarkRequire => "benchRequire",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RegisteredAccess {
    Public,
    Authenticated,
    Require(Vec<String>),
    Any(Vec<String>),
    All(Vec<String>),
}

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum ContractError {
    #[error("capability collection must not be empty")]
    EmptyCapabilities,
    #[error("capability must not be empty")]
    EmptyCapability,
    #[error("route already registered for {method} {path}")]
    RouteConflict { method: Method, path: String },
    #[error("operation id already registered: {0}")]
    OperationIdConflict(String),
}
