use super::*;

pub(super) fn response_schema(operation: &OperationDescriptor) -> &'static str {
    match operation {
        OperationDescriptor::Login => "ApiResponseLoginResp",
        OperationDescriptor::CurrentAdminUser | OperationDescriptor::UpdateAccountProfile => {
            "ApiResponseUserInfoResp"
        }
        OperationDescriptor::Logout
        | OperationDescriptor::ChangeAccountPassword
        | OperationDescriptor::CreateRole
        | OperationDescriptor::UpdateRole
        | OperationDescriptor::DeleteRole
        | OperationDescriptor::DeleteMenu
        | OperationDescriptor::DeleteUser => "ApiResponseUnit",
        OperationDescriptor::UpdateAccountAvatar => "ApiResponseString",
        OperationDescriptor::GetDashboardStats => "ApiResponseStatsResp",
        OperationDescriptor::ListManageLogs => "ApiResponseLogItemRespList",
        OperationDescriptor::ListManageTasks => "ApiResponseTaskItemList",
        OperationDescriptor::ListTaskRuns => "ApiResponseTaskRunItemList",
        OperationDescriptor::RunTask => "ApiResponseTaskRunItem",
        OperationDescriptor::ListDeployments => "ApiResponseDeploymentItemList",
        OperationDescriptor::UploadDeployment
        | OperationDescriptor::GetDeployment
        | OperationDescriptor::ExpireDeployment
        | OperationDescriptor::DeleteDeployment => "ApiResponseDeploymentItem",
        OperationDescriptor::CleanupDeployments => "ApiResponseI64",
        OperationDescriptor::DeployVersion
        | OperationDescriptor::UpdateUserPassword
        | OperationDescriptor::UpdateUserStatus
        | OperationDescriptor::RevokeUserSessions => "ApiResponseBool",
        OperationDescriptor::ListMenus | OperationDescriptor::ListModuleMenuInventory => {
            "ApiResponseMenuItemRespList"
        }
        OperationDescriptor::UpdateMenu => "ApiResponseI64",
        OperationDescriptor::GetMenuOptions => "ApiResponseMenuOptionRespList",
        OperationDescriptor::ListRoles => "ApiResponseRoleItemRespList",
        OperationDescriptor::GetRoleOptions => "ApiResponseRoleOptionRespList",
        OperationDescriptor::GetStatusOverview => "ApiResponseSystemStatusOverview",
        OperationDescriptor::ListModuleLogs => "ApiResponseModuleLogFileRespList",
        OperationDescriptor::TailModuleLog => "ApiResponseModuleLogTailResp",
        OperationDescriptor::PreviewModuleLogCleanup => "ApiResponseModuleLogCleanupPreviewResp",
        OperationDescriptor::ConfirmModuleLogCleanup => "ApiResponseModuleLogCleanupResultResp",
        OperationDescriptor::ListUsers => "ApiResponseUserItemRespList",
        OperationDescriptor::CreateAdminUser | OperationDescriptor::UpdateUser => "ApiResponseI64",
        OperationDescriptor::GetUserOptions | OperationDescriptor::GetUserStatusOptions => {
            "ApiResponseUserOptionRespList"
        }
        OperationDescriptor::ListModules | OperationDescriptor::UpdateModuleEnabled => {
            "ApiResponseModuleStatusResponseList"
        }
        OperationDescriptor::GetModuleNavigation => "ApiResponseRuntimeMenuResponseList",
        OperationDescriptor::GetDashboardModules => "ApiResponseModuleHealthResponseList",
        OperationDescriptor::ListNotifications => "ApiResponseInboxListResponse",
        OperationDescriptor::GetNotificationUnreadCount => "ApiResponseUnreadCountResponse",
        OperationDescriptor::GetNotification => "ApiResponseNotificationItem",
        OperationDescriptor::ReadNotification => "ApiResponseReadResponse",
        OperationDescriptor::ReadAllNotifications => "ApiResponseReadAllResponse",
        OperationDescriptor::ExportManageLogs
        | OperationDescriptor::BackupModuleLogs
        | OperationDescriptor::ContractPublic
        | OperationDescriptor::ContractAny
        | OperationDescriptor::ContractAll
        | OperationDescriptor::BenchmarkRequire => "ApiResponseJson",
    }
}

pub(super) fn response_schemas() -> Vec<(&'static str, serde_json::Value)> {
    let reference =
        |name: &str| serde_json::json!({ "$ref": format!("#/components/schemas/{name}") });
    let list = |name: &str| serde_json::json!({ "type": "array", "items": reference(name) });
    let schemas = vec![
        ("ApiResponseLoginResp", reference("LoginResp")),
        ("ApiResponseUserInfoResp", reference("UserInfoResp")),
        ("ApiResponseString", serde_json::json!({ "type": "string" })),
        ("ApiResponseBool", serde_json::json!({ "type": "boolean" })),
        ("ApiResponseI64", serde_json::json!({ "type": "integer", "format": "int64" })),
        ("ApiResponseStatsResp", reference("StatsResp")),
        ("ApiResponseLogItemRespList", list("LogItemResp")),
        ("ApiResponseTaskItemList", list("TaskItem")),
        ("ApiResponseTaskRunItemList", list("TaskRunItem")),
        ("ApiResponseTaskRunItem", reference("TaskRunItem")),
        ("ApiResponseDeploymentItemList", list("DeploymentItem")),
        ("ApiResponseDeploymentItem", reference("DeploymentItem")),
        ("ApiResponseMenuItemRespList", list("MenuItemResp")),
        ("ApiResponseMenuOptionRespList", list("MenuOptionResp")),
        ("ApiResponseRoleItemRespList", list("RoleItemResp")),
        ("ApiResponseRoleOptionRespList", list("RoleOptionResp")),
        ("ApiResponseSystemStatusOverview", reference("SystemStatusOverview")),
        ("ApiResponseModuleLogFileRespList", list("ModuleLogFileResp")),
        ("ApiResponseModuleLogTailResp", reference("ModuleLogTailResp")),
        ("ApiResponseModuleLogCleanupPreviewResp", reference("ModuleLogCleanupPreviewResp")),
        ("ApiResponseModuleLogCleanupResultResp", reference("ModuleLogCleanupResultResp")),
        ("ApiResponseUserItemRespList", list("UserItemResp")),
        ("ApiResponseUserOptionRespList", list("OptionItem")),
        ("ApiResponseModuleStatusResponseList", list("ModuleStatusResponse")),
        ("ApiResponseRuntimeMenuResponseList", list("RuntimeMenuResponse")),
        ("ApiResponseModuleHealthResponseList", list("ModuleHealthResponse")),
        ("ApiResponseUnit", serde_json::json!({ "type": "null" })),
        ("ApiResponseJson", serde_json::json!({ "type": "object", "additionalProperties": true })),
    ];
    #[cfg(feature = "notifications")]
    {
        let mut schemas = schemas;
        schemas.extend([
            ("ApiResponseInboxListResponse", reference("InboxListResponse")),
            ("ApiResponseUnreadCountResponse", reference("UnreadCountResponse")),
            ("ApiResponseNotificationItem", reference("NotificationItem")),
            ("ApiResponseReadResponse", reference("ReadResponse")),
            ("ApiResponseReadAllResponse", reference("ReadAllResponse")),
        ]);
        schemas
    }
    #[cfg(not(feature = "notifications"))]
    {
        schemas
    }
}

pub(super) fn request_schema(operation: &OperationDescriptor) -> Option<&'static str> {
    match operation {
        OperationDescriptor::Login => Some("LoginRequest"),
        OperationDescriptor::UpdateAccountProfile => Some("UpdateAccountProfileRequest"),
        OperationDescriptor::ChangeAccountPassword => Some("ChangeAccountPasswordRequest"),
        OperationDescriptor::ExpireDeployment => Some("ExpireVersionRequest"),
        OperationDescriptor::UpdateMenu => Some("UpdateMenuPayload"),
        OperationDescriptor::CreateRole => Some("CreateRoleRequest"),
        OperationDescriptor::UpdateRole => Some("UpdateRolePayload"),
        OperationDescriptor::CreateAdminUser => Some("CreateUserRequest"),
        OperationDescriptor::UpdateUser => Some("UpdateUserPayload"),
        OperationDescriptor::UpdateUserPassword => Some("UpdateUserPasswordPayload"),
        OperationDescriptor::UpdateUserStatus => Some("UpdateUserStatusPayload"),
        OperationDescriptor::UpdateModuleEnabled => Some("UpdateModuleRequest"),
        OperationDescriptor::BackupModuleLogs => Some("ModuleLogBackupRequest"),
        OperationDescriptor::ConfirmModuleLogCleanup => Some("ModuleLogCleanupConfirmRequest"),
        OperationDescriptor::ReadAllNotifications => Some("ReadAllRequest"),
        _ => None,
    }
}

pub(super) fn multipart_request_schema(operation: &OperationDescriptor) -> Option<&'static str> {
    match operation {
        OperationDescriptor::UpdateAccountAvatar => Some("AvatarUpload"),
        OperationDescriptor::UploadDeployment => Some("DeploymentUpload"),
        _ => None,
    }
}

pub(super) fn json_request_body(schema: &str) -> utoipa::openapi::request_body::RequestBody {
    RequestBodyBuilder::new()
        .required(Some(utoipa::openapi::Required::True))
        .content("application/json", Content::new(Some(Ref::from_schema_name(schema))))
        .build()
}

pub(super) fn multipart_schema(
    fields: &[(&str, &str, bool)],
) -> utoipa::openapi::RefOr<utoipa::openapi::Schema> {
    let properties = fields.iter().map(|(name, content_type, _required)| {
        let schema = if *name == "component" {
            serde_json::json!({ "$ref": "#/components/schemas/DeployComponent" })
        } else if content_type.starts_with("image/") || *content_type == "application/octet-stream" {
            serde_json::json!({ "type": "string", "format": "binary", "contentMediaType": content_type })
        } else {
            serde_json::json!({ "type": "string" })
        };
        (name.to_string(), schema)
    }).collect::<serde_json::Map<_, _>>();
    let required = fields
        .iter()
        .filter(|(_, _, required)| *required)
        .map(|(name, _, _)| serde_json::Value::String((*name).to_string()))
        .collect::<Vec<_>>();
    serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required
    }))
    .expect("static multipart schema")
}

pub(super) fn query_parameters(
    operation: &OperationDescriptor,
) -> Vec<utoipa::openapi::path::Parameter> {
    let query = || Some(ParameterIn::Query);
    match operation {
        OperationDescriptor::ListManageLogs | OperationDescriptor::ExportManageLogs => {
            <LogQuery as utoipa::IntoParams>::into_params(query)
        }
        OperationDescriptor::ListTaskRuns => {
            <TaskRunQuery as utoipa::IntoParams>::into_params(query)
        }
        OperationDescriptor::ListDeployments => {
            <ListDeploymentsQuery as utoipa::IntoParams>::into_params(query)
        }
        OperationDescriptor::CleanupDeployments => {
            <CleanupDeploymentsQuery as utoipa::IntoParams>::into_params(query)
        }
        OperationDescriptor::ListMenus => <MenuQuery as utoipa::IntoParams>::into_params(query),
        OperationDescriptor::GetMenuOptions => {
            <crate::common::api::OptionsQuery as utoipa::IntoParams>::into_params(query)
        }
        OperationDescriptor::ListRoles => <RoleQuery as utoipa::IntoParams>::into_params(query),
        OperationDescriptor::GetRoleOptions => {
            <crate::common::api::OptionsQuery as utoipa::IntoParams>::into_params(query)
        }
        OperationDescriptor::ListUsers => <UserQuery as utoipa::IntoParams>::into_params(query),
        OperationDescriptor::GetUserOptions => {
            <UserOptionsQuery as utoipa::IntoParams>::into_params(query)
        }
        OperationDescriptor::ListModuleLogs => {
            <crate::features::system::status::logs::types::ModuleLogListQuery as utoipa::IntoParams>::into_params(query)
        }
        OperationDescriptor::TailModuleLog => {
            <crate::features::system::status::logs::types::ModuleLogTailQuery as utoipa::IntoParams>::into_params(query)
        }
        OperationDescriptor::ListNotifications => {
            vec![
                ParameterBuilder::new()
                    .name("cursor")
                    .parameter_in(ParameterIn::Query)
                    .required(utoipa::openapi::Required::False)
                    .schema(Some(ObjectBuilder::new().schema_type(Type::String).build()))
                    .build(),
                ParameterBuilder::new()
                    .name("limit")
                    .parameter_in(ParameterIn::Query)
                    .required(utoipa::openapi::Required::False)
                    .schema(Some(
                        ObjectBuilder::new()
                            .schema_type(Type::Integer)
                            .minimum(Some(1))
                            .maximum(Some(100))
                            .build(),
                    ))
                    .build(),
                ParameterBuilder::new()
                    .name("unreadOnly")
                    .parameter_in(ParameterIn::Query)
                    .required(utoipa::openapi::Required::False)
                    .schema(Some(ObjectBuilder::new().schema_type(Type::Boolean).build()))
                    .build(),
            ]
        }
        _ => Vec::new(),
    }
}

pub(super) fn json_success_response(schema: &str) -> utoipa::openapi::Response {
    ResponseBuilder::new()
        .description("Success")
        .content("application/json", Content::new(Some(Ref::from_schema_name(schema))))
        .build()
}

pub(super) fn plain_response(description: &str) -> utoipa::openapi::Response {
    ResponseBuilder::new()
        .description(description)
        .content("text/plain", Content::new(None::<Ref>))
        .build()
}

pub(super) fn error_schema() -> utoipa::openapi::RefOr<utoipa::openapi::Schema> {
    serde_json::from_value(serde_json::json!({
        "type": "object",
        "required": ["code", "message", "data"],
        "properties": {
            "code": { "type": "integer", "format": "int32" },
            "message": { "type": "string" },
            "data": { "type": "null" }
        }
    }))
    .expect("static error envelope schema")
}

pub(super) fn envelope_schema(
    data: serde_json::Value,
) -> utoipa::openapi::RefOr<utoipa::openapi::Schema> {
    serde_json::from_value(serde_json::json!({
        "type": "object",
        "required": ["code", "message", "data"],
        "properties": {
            "code": { "type": "integer" },
            "message": { "type": "string" },
            "data": data,
            "total": { "type": ["integer", "null"], "format": "int64" }
        }
    }))
    .expect("static envelope schema")
}
