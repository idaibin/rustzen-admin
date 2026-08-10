//! Programmatic OpenAPI output derived from Admin's registered route contracts.

use crate::{
    common::api::OptionItem,
    features::{
        account::types::{ChangeAccountPasswordRequest, UpdateAccountProfileRequest},
        auth::types::{LoginRequest, LoginResp, UserInfoResp},
        dashboard::types::StatsResp,
        manage::{
            deploy::types::{
                CleanupDeploymentsQuery, DeployComponent, DeploymentItem, ExpireVersionRequest,
                ListDeploymentsQuery,
            },
            log::types::{LogItemResp, LogQuery},
            task::types::{
                TaskItem, TaskRunItem, TaskRunQuery, TaskRunStatus, TaskSchedule, TaskTriggerType,
            },
        },
        modules::types::{
            ModuleHealthResponse, ModuleStatusResponse, RuntimeMenuResponse, UpdateModuleRequest,
        },
        system::{
            menu::types::{MenuItemResp, MenuOptionResp, MenuQuery, UpdateMenuPayload},
            role::types::{
                CreateRoleRequest, RoleItemResp, RoleOptionResp, RoleQuery, UpdateRolePayload,
            },
            status::logs::types::{
                ModuleLogBackupRequest, ModuleLogCleanupCandidate, ModuleLogCleanupConfirmRequest,
                ModuleLogCleanupPreviewResp, ModuleLogCleanupResultResp, ModuleLogFileResp,
                ModuleLogFileSelector, ModuleLogItemFailure, ModuleLogTailResp,
            },
            status::types::{
                CpuResourceStatus, DirectoryStorageItem, DiskResourceStatus, LocalResourceStatus,
                MemoryResourceStatus, SqliteStorageStatus, SystemStatusOverview,
                SystemStorageStatus,
            },
            user::types::{
                CreateUserRequest, UpdateUserPasswordPayload, UpdateUserPayload,
                UpdateUserStatusPayload, UserItemResp, UserOptionsQuery, UserQuery, UserRoleResp,
            },
        },
    },
    infra::{
        app::documented_all_contracts,
        contract::{OperationDescriptor, RegisteredAccess},
    },
};
use utoipa::openapi::{
    ComponentsBuilder, Content, Header, Info, OpenApi, OpenApiBuilder, Paths, Ref, ResponseBuilder,
    SchemaFormat,
    extensions::ExtensionsBuilder,
    path::{HttpMethod, Operation, OperationBuilder},
    path::{ParameterBuilder, ParameterIn},
    request_body::RequestBodyBuilder,
    schema::{ObjectBuilder, Type},
    security::{Http, HttpAuthScheme, SecurityRequirement, SecurityScheme},
};

pub fn document() -> Result<OpenApi, crate::infra::contract::ContractError> {
    let contracts = documented_all_contracts();
    let mut paths = Paths::new();
    for contract in contracts {
        let operation = operation_for(&contract);
        let method = match contract.method {
            axum::http::Method::GET => HttpMethod::Get,
            axum::http::Method::POST => HttpMethod::Post,
            axum::http::Method::PUT => HttpMethod::Put,
            axum::http::Method::PATCH => HttpMethod::Patch,
            axum::http::Method::DELETE => HttpMethod::Delete,
            _ => unreachable!("unsupported method in the Admin API contract"),
        };
        paths.add_path_operation(&contract.path, vec![method], operation);
    }
    let mut components = ComponentsBuilder::new()
        .schema_from::<LoginRequest>()
        .schema_from::<LoginResp>()
        .schema_from::<UserInfoResp>()
        .schema_from::<CreateUserRequest>()
        .schema_from::<ChangeAccountPasswordRequest>()
        .schema_from::<UpdateAccountProfileRequest>()
        .schema_from::<StatsResp>()
        .schema_from::<LogItemResp>()
        .schema_from::<TaskItem>()
        .schema_from::<TaskRunItem>()
        .schema_from::<TaskSchedule>()
        .schema_from::<TaskTriggerType>()
        .schema_from::<TaskRunStatus>()
        .schema_from::<DeploymentItem>()
        .schema_from::<DeployComponent>()
        .schema_from::<ExpireVersionRequest>()
        .schema_from::<MenuItemResp>()
        .schema_from::<MenuOptionResp>()
        .schema_from::<UpdateMenuPayload>()
        .schema_from::<CreateRoleRequest>()
        .schema_from::<RoleItemResp>()
        .schema_from::<RoleOptionResp>()
        .schema_from::<UpdateRolePayload>()
        .schema_from::<SystemStatusOverview>()
        .schema_from::<SystemStorageStatus>()
        .schema_from::<SqliteStorageStatus>()
        .schema_from::<DirectoryStorageItem>()
        .schema_from::<LocalResourceStatus>()
        .schema_from::<CpuResourceStatus>()
        .schema_from::<MemoryResourceStatus>()
        .schema_from::<DiskResourceStatus>()
        .schema_from::<UserItemResp>()
        .schema_from::<UserRoleResp>()
        .schema_from::<UpdateUserPayload>()
        .schema_from::<UpdateUserPasswordPayload>()
        .schema_from::<UpdateUserStatusPayload>()
        .schema_from::<OptionItem<i64>>()
        .schema_from::<ModuleStatusResponse>()
        .schema_from::<ModuleHealthResponse>()
        .schema_from::<RuntimeMenuResponse>()
        .schema_from::<UpdateModuleRequest>()
        .schema_from::<ModuleLogFileSelector>()
        .schema_from::<ModuleLogBackupRequest>()
        .schema_from::<ModuleLogCleanupConfirmRequest>()
        .schema_from::<ModuleLogFileResp>()
        .schema_from::<ModuleLogTailResp>()
        .schema_from::<ModuleLogCleanupCandidate>()
        .schema_from::<ModuleLogItemFailure>()
        .schema_from::<ModuleLogCleanupPreviewResp>()
        .schema_from::<ModuleLogCleanupResultResp>()
        .build();
    if let Some(option_item) = components.schemas.get("OptionItem").cloned() {
        components.schemas.insert("OptionItem_i64".into(), option_item);
    }
    components.schemas.insert(
        "i64".into(),
        serde_json::from_value(serde_json::json!({ "type": "integer", "format": "int64" }))
            .expect("static scalar schema"),
    );
    for (name, data) in response_schemas() {
        components.schemas.insert(name.to_string(), envelope_schema(data));
    }
    components.schemas.insert("ApiErrorResponse".into(), error_schema());
    components
        .schemas
        .insert("ApiResponseUnit".into(), envelope_schema(serde_json::json!({"type": "null"})));
    components
        .schemas
        .insert("AvatarUpload".into(), multipart_schema(&[("file", "image/*", true)]));
    components.schemas.insert(
        "DeploymentUpload".into(),
        multipart_schema(&[
            ("component", "text/plain", true),
            ("version", "text/plain", true),
            ("arch", "text/plain", false),
            ("notes", "text/plain", false),
            ("file", "application/octet-stream", true),
        ]),
    );
    components
        .add_security_scheme("bearerAuth", SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)));
    Ok(OpenApiBuilder::new()
        .info(Info::new("RustZen Admin contract trial", env!("CARGO_PKG_VERSION")))
        .paths(paths)
        .components(Some(components))
        .build())
}

fn operation_for(contract: &crate::infra::contract::RouteContract) -> Operation {
    let mut operation =
        OperationBuilder::new().operation_id(Some(contract.operation.operation_id()));
    operation = match contract.operation {
        OperationDescriptor::Login => operation
            .response("200", json_success_response(response_schema(&contract.operation)))
            .request_body(Some(json_request_body("LoginRequest"))),
        OperationDescriptor::CurrentAdminUser => {
            operation.response("200", json_success_response("ApiResponseUserInfoResp"))
        }
        OperationDescriptor::CreateAdminUser => operation
            .response("200", json_success_response("ApiResponseI64"))
            .request_body(Some(
                RequestBodyBuilder::new()
                    .required(Some(utoipa::openapi::Required::True))
                    .content(
                        "application/json",
                        Content::new(Some(Ref::from_schema_name("CreateUserRequest"))),
                    )
                    .build(),
            ))
            .response("400", plain_response("Malformed or invalid JSON"))
            .response("415", plain_response("Content-Type must be application/json"))
            .response("422", plain_response("DTO deserialization failed"))
            .response("404", ResponseBuilder::new().description("Role not found").build())
            .response(
                "409",
                ResponseBuilder::new().description("Username or email conflict").build(),
            ),
        OperationDescriptor::ExportManageLogs => operation.response(
            "200",
            ResponseBuilder::new()
                .description("CSV export")
                .content(
                    "text/csv",
                    Content::new(Some(ObjectBuilder::new().schema_type(Type::String).build())),
                )
                .build(),
        ),
        OperationDescriptor::BackupModuleLogs => operation.response(
            "200",
            ResponseBuilder::new()
                .description("Bounded module-log tar archive")
                .content(
                    "application/x-tar",
                    Content::new(Some(
                        ObjectBuilder::new()
                            .schema_type(Type::String)
                            .format(Some(SchemaFormat::Custom("binary".into())))
                            .build(),
                    )),
                )
                .header(
                    "content-disposition",
                    Header::new(ObjectBuilder::new().schema_type(Type::String).build()),
                )
                .header(
                    "x-rustzen-archive-sha256",
                    Header::new(ObjectBuilder::new().schema_type(Type::String).build()),
                )
                .header(
                    "x-rustzen-archive-file-count",
                    Header::new(ObjectBuilder::new().schema_type(Type::Integer).build()),
                )
                .build(),
        ),
        OperationDescriptor::Logout
        | OperationDescriptor::UpdateAccountAvatar
        | OperationDescriptor::UpdateAccountProfile
        | OperationDescriptor::ChangeAccountPassword
        | OperationDescriptor::GetDashboardStats
        | OperationDescriptor::ListManageLogs
        | OperationDescriptor::ListManageTasks
        | OperationDescriptor::ListTaskRuns
        | OperationDescriptor::RunTask
        | OperationDescriptor::ListDeployments
        | OperationDescriptor::UploadDeployment
        | OperationDescriptor::CleanupDeployments
        | OperationDescriptor::GetDeployment
        | OperationDescriptor::ExpireDeployment
        | OperationDescriptor::DeleteDeployment
        | OperationDescriptor::DeployVersion
        | OperationDescriptor::ListMenus
        | OperationDescriptor::ListModuleMenuInventory
        | OperationDescriptor::UpdateMenu
        | OperationDescriptor::DeleteMenu
        | OperationDescriptor::GetMenuOptions
        | OperationDescriptor::ListRoles
        | OperationDescriptor::CreateRole
        | OperationDescriptor::UpdateRole
        | OperationDescriptor::DeleteRole
        | OperationDescriptor::GetRoleOptions
        | OperationDescriptor::GetStatusOverview
        | OperationDescriptor::ListModuleLogs
        | OperationDescriptor::TailModuleLog
        | OperationDescriptor::PreviewModuleLogCleanup
        | OperationDescriptor::ConfirmModuleLogCleanup
        | OperationDescriptor::ListUsers
        | OperationDescriptor::UpdateUser
        | OperationDescriptor::DeleteUser
        | OperationDescriptor::GetUserOptions
        | OperationDescriptor::GetUserStatusOptions
        | OperationDescriptor::UpdateUserPassword
        | OperationDescriptor::UpdateUserStatus
        | OperationDescriptor::ListModules
        | OperationDescriptor::UpdateModuleEnabled
        | OperationDescriptor::GetModuleNavigation
        | OperationDescriptor::GetDashboardModules => {
            operation.response("200", json_success_response(response_schema(&contract.operation)))
        }
        OperationDescriptor::ContractPublic
        | OperationDescriptor::ContractAny
        | OperationDescriptor::ContractAll
        | OperationDescriptor::BenchmarkRequire => {
            operation.response("204", ResponseBuilder::new().description("No content").build())
        }
    };
    if let Some(schema) = request_schema(&contract.operation) {
        operation = operation.request_body(Some(json_request_body(schema)));
    } else if let Some(schema) = multipart_request_schema(&contract.operation) {
        operation = operation.request_body(Some(
            RequestBodyBuilder::new()
                .required(Some(utoipa::openapi::Required::True))
                .content("multipart/form-data", Content::new(Some(Ref::from_schema_name(schema))))
                .build(),
        ));
    }
    for segment in contract.path.split('/') {
        let Some(name) = segment.strip_prefix('{').and_then(|value| value.strip_suffix('}')) else {
            continue;
        };
        let schema_type = if name == "id" { Type::Integer } else { Type::String };
        operation = operation.parameter(
            ParameterBuilder::new()
                .name(name)
                .parameter_in(ParameterIn::Path)
                .required(utoipa::openapi::Required::True)
                .schema(Some(ObjectBuilder::new().schema_type(schema_type).build()))
                .build(),
        );
    }
    for parameter in query_parameters(&contract.operation) {
        operation = operation.parameter(parameter);
    }
    match &contract.access {
        RegisteredAccess::Public => {
            operation = operation.securities(Some(Vec::<SecurityRequirement>::new()))
        }
        RegisteredAccess::Authenticated => {
            operation = operation
                .security(SecurityRequirement::new("bearerAuth", Vec::<String>::new()))
                .response("401", json_error_response("Authentication required"))
        }
        RegisteredAccess::Require(codes)
        | RegisteredAccess::Any(codes)
        | RegisteredAccess::All(codes) => {
            let mode = match contract.access {
                RegisteredAccess::Require(_) => "require",
                RegisteredAccess::Any(_) => "any",
                _ => "all",
            };
            operation = operation
                .security(SecurityRequirement::new("bearerAuth", Vec::<String>::new()))
                .response("401", json_error_response("Authentication required"))
                .response("403", json_error_response("Missing capability"))
                .extensions(Some(
                    ExtensionsBuilder::new()
                        .add(
                            "rustzen-authorization",
                            serde_json::json!({"mode": mode, "capabilities": codes}),
                        )
                        .build(),
                ));
        }
    }
    for spec in error_specs(&contract.operation) {
        operation = operation.response(spec.status, error_response(spec));
    }
    operation.build()
}

#[derive(Clone, Copy)]
struct ErrorSpec {
    status: &'static str,
    description: &'static str,
    json: bool,
    text_plain: bool,
}

const fn json_error(status: &'static str, description: &'static str) -> ErrorSpec {
    ErrorSpec { status, description, json: true, text_plain: false }
}

const fn extractor_error(status: &'static str, description: &'static str) -> ErrorSpec {
    ErrorSpec { status, description, json: false, text_plain: true }
}

const fn json_and_extractor_error(status: &'static str, description: &'static str) -> ErrorSpec {
    ErrorSpec { status, description, json: true, text_plain: true }
}

fn error_specs(operation: &OperationDescriptor) -> Vec<ErrorSpec> {
    use OperationDescriptor::*;
    match operation {
        Login => vec![
            json_and_extractor_error("400", "Invalid JSON or pending/locked account"),
            extractor_error("415", "Content-Type must be application/json"),
            extractor_error("422", "DTO deserialization failed"),
            json_error("401", "Invalid username or password"),
            json_error("403", "Account is disabled"),
            json_error("500", "Internal server error"),
        ],
        CurrentAdminUser => vec![
            json_error("404", "Current user not found"),
            json_error("500", "Internal server error"),
        ],
        Logout => Vec::new(),
        UpdateAccountAvatar => vec![
            json_and_extractor_error("400", "Invalid avatar upload"),
            extractor_error("413", "Avatar upload exceeds the body limit"),
            json_error("500", "Internal server error"),
        ],
        UpdateAccountProfile => json_body_errors(&[
            json_error("409", "Email already exists"),
            json_error("500", "Internal server error"),
        ]),
        ChangeAccountPassword => json_body_errors(&[
            json_error("404", "User not found"),
            json_error("500", "Internal server error"),
        ]),
        GetDashboardStats => vec![json_error("500", "Internal server error")],
        ListManageLogs => vec![
            extractor_error("400", "Invalid query parameters"),
            json_error("500", "Internal server error"),
        ],
        ExportManageLogs => vec![
            extractor_error("400", "Invalid query parameters"),
            json_error("500", "Internal server error"),
        ],
        ListManageTasks => vec![json_error("500", "Internal server error")],
        ListTaskRuns => vec![
            extractor_error("400", "Invalid query parameters"),
            json_error("404", "Task not found"),
            json_error("500", "Internal server error"),
        ],
        RunTask => vec![
            json_error("400", "Task cannot be run"),
            json_error("404", "Task not found"),
            json_error("500", "Internal server error"),
        ],
        ListDeployments => vec![
            extractor_error("400", "Invalid query parameters"),
            json_error("500", "Internal server error"),
        ],
        UploadDeployment => vec![
            json_and_extractor_error("400", "Invalid deployment upload"),
            extractor_error("413", "Deployment upload exceeds the body limit"),
            json_error("500", "Internal server error"),
        ],
        CleanupDeployments => vec![
            extractor_error("400", "Invalid query parameters"),
            json_error("500", "Internal server error"),
        ],
        GetDeployment => vec![
            extractor_error("400", "Invalid deployment id"),
            json_error("404", "Deployment not found"),
            json_error("500", "Internal server error"),
        ],
        ExpireDeployment => json_body_errors(&[
            json_error("404", "Deployment not found"),
            json_error("500", "Internal server error"),
        ]),
        DeleteDeployment | DeployVersion => vec![
            json_and_extractor_error("400", "Deployment operation is invalid"),
            json_error("404", "Deployment not found"),
            json_error("500", "Internal server error"),
        ],
        ListMenus => vec![
            extractor_error("400", "Invalid query parameters"),
            json_error("500", "Internal server error"),
        ],
        ListModuleMenuInventory => vec![json_error("500", "Internal server error")],
        UpdateMenu => json_body_errors(&[
            json_error("404", "Menu not found"),
            json_error("500", "Internal server error"),
        ]),
        DeleteMenu => vec![
            json_and_extractor_error("400", "Menu operation is invalid"),
            json_error("404", "Menu not found"),
            json_error("500", "Internal server error"),
        ],
        GetMenuOptions => vec![
            extractor_error("400", "Invalid query parameters"),
            json_error("500", "Internal server error"),
        ],
        ListRoles => vec![
            extractor_error("400", "Invalid query parameters"),
            json_error("500", "Internal server error"),
        ],
        CreateRole | UpdateRole => json_body_errors(&[
            json_error("404", "Role or menu not found"),
            json_error("500", "Internal server error"),
        ]),
        DeleteRole => vec![
            json_and_extractor_error("400", "Role operation is invalid"),
            json_error("404", "Role not found"),
            json_error("500", "Internal server error"),
        ],
        GetRoleOptions => vec![
            extractor_error("400", "Invalid query parameters"),
            json_error("500", "Internal server error"),
        ],
        GetStatusOverview => vec![json_error("400", "Status collection failed")],
        ListModuleLogs | TailModuleLog | PreviewModuleLogCleanup => vec![
            extractor_error("400", "Invalid module-log query or filesystem scope"),
            json_error("500", "Internal server error"),
        ],
        BackupModuleLogs | ConfirmModuleLogCleanup => vec![
            json_and_extractor_error("400", "Invalid module-log request or filesystem scope"),
            extractor_error("415", "Content-Type must be application/json"),
            extractor_error("422", "DTO deserialization failed"),
            json_error("500", "Internal server error"),
        ],
        ListUsers => vec![
            extractor_error("400", "Invalid query parameters"),
            json_error("500", "Internal server error"),
        ],
        CreateAdminUser | UpdateUser => json_body_errors(&[
            json_error("404", "Role or user not found"),
            json_error("409", "Username or email already exists"),
            json_error("500", "Internal server error"),
        ]),
        DeleteUser => vec![
            json_and_extractor_error("400", "User operation is invalid"),
            json_error("404", "User not found"),
            json_error("500", "Internal server error"),
        ],
        GetUserOptions => vec![
            extractor_error("400", "Invalid query parameters"),
            json_error("500", "Internal server error"),
        ],
        GetUserStatusOptions => Vec::new(),
        UpdateUserPassword => json_body_errors(&[
            json_error("404", "User not found"),
            json_error("500", "Internal server error"),
        ]),
        UpdateUserStatus => json_body_errors(&[
            json_error("404", "User not found"),
            json_error("500", "Internal server error"),
        ]),
        ListModules | GetDashboardModules => Vec::new(),
        UpdateModuleEnabled => json_body_errors(&[
            json_error("404", "Module not found"),
            json_error("500", "Internal server error"),
        ]),
        GetModuleNavigation => vec![json_error("500", "Internal server error")],
        ContractPublic | ContractAny | ContractAll | BenchmarkRequire => Vec::new(),
    }
}

fn json_body_errors(additional: &[ErrorSpec]) -> Vec<ErrorSpec> {
    let mut specs = vec![
        json_and_extractor_error("400", "Invalid JSON or request body"),
        extractor_error("415", "Content-Type must be application/json"),
        extractor_error("422", "DTO deserialization failed"),
    ];
    specs.extend_from_slice(additional);
    specs
}

fn response_schema(operation: &OperationDescriptor) -> &'static str {
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
        | OperationDescriptor::UpdateUserStatus => "ApiResponseBool",
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
        OperationDescriptor::ExportManageLogs
        | OperationDescriptor::BackupModuleLogs
        | OperationDescriptor::ContractPublic
        | OperationDescriptor::ContractAny
        | OperationDescriptor::ContractAll
        | OperationDescriptor::BenchmarkRequire => "ApiResponseJson",
    }
}

fn response_schemas() -> Vec<(&'static str, serde_json::Value)> {
    let reference =
        |name: &str| serde_json::json!({ "$ref": format!("#/components/schemas/{name}") });
    let list = |name: &str| serde_json::json!({ "type": "array", "items": reference(name) });
    vec![
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
    ]
}

fn request_schema(operation: &OperationDescriptor) -> Option<&'static str> {
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
        _ => None,
    }
}

fn multipart_request_schema(operation: &OperationDescriptor) -> Option<&'static str> {
    match operation {
        OperationDescriptor::UpdateAccountAvatar => Some("AvatarUpload"),
        OperationDescriptor::UploadDeployment => Some("DeploymentUpload"),
        _ => None,
    }
}

fn json_request_body(schema: &str) -> utoipa::openapi::request_body::RequestBody {
    RequestBodyBuilder::new()
        .required(Some(utoipa::openapi::Required::True))
        .content("application/json", Content::new(Some(Ref::from_schema_name(schema))))
        .build()
}

fn multipart_schema(
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

fn query_parameters(operation: &OperationDescriptor) -> Vec<utoipa::openapi::path::Parameter> {
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
        _ => Vec::new(),
    }
}

fn json_success_response(schema: &str) -> utoipa::openapi::Response {
    ResponseBuilder::new()
        .description("Success")
        .content("application/json", Content::new(Some(Ref::from_schema_name(schema))))
        .build()
}

fn plain_response(description: &str) -> utoipa::openapi::Response {
    ResponseBuilder::new()
        .description(description)
        .content("text/plain", Content::new(None::<Ref>))
        .build()
}

fn json_error_response(description: &str) -> utoipa::openapi::Response {
    ResponseBuilder::new()
        .description(description)
        .content("application/json", Content::new(Some(Ref::from_schema_name("ApiErrorResponse"))))
        .build()
}

fn error_response(spec: ErrorSpec) -> utoipa::openapi::Response {
    let mut response = ResponseBuilder::new().description(spec.description);
    if spec.json {
        response = response.content(
            "application/json",
            Content::new(Some(Ref::from_schema_name("ApiErrorResponse"))),
        );
    }
    if spec.text_plain {
        response = response.content("text/plain", Content::new(None::<Ref>));
    }
    response.build()
}

fn error_schema() -> utoipa::openapi::RefOr<utoipa::openapi::Schema> {
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

fn envelope_schema(data: serde_json::Value) -> utoipa::openapi::RefOr<utoipa::openapi::Schema> {
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

pub fn normalized_json() -> Result<String, Box<dyn std::error::Error>> {
    Ok(serde_json::to_string_pretty(&document()?)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::contract::{RegisteredAccess, RouteContract};
    #[test]
    fn generated_document_matches_registered_paths_and_auth_metadata() {
        let value: serde_json::Value = serde_json::from_str(&normalized_json().unwrap()).unwrap();
        assert!(value["paths"]["/api/auth/me"]["get"]["security"].is_array());
        assert_eq!(
            value["paths"]["/api/system/users"]["post"]["x-rustzen-authorization"]["capabilities"]
                [0],
            "system:user:create"
        );
        assert_eq!(
            value["paths"]["/api/system/users"]["post"]["responses"]["200"]["content"]["application/json"]
                ["schema"]["$ref"],
            "#/components/schemas/ApiResponseI64"
        );
        assert_eq!(
            value["components"]["schemas"]["ApiResponseUserInfoResp"]["properties"]["data"]["$ref"],
            "#/components/schemas/UserInfoResp"
        );
        assert!(value["paths"]["/api/system/users"]["post"]["responses"]["415"].is_object());
        assert!(value["paths"]["/api/system/users"]["post"]["responses"]["422"]["content"]["text/plain"].is_object());
    }

    #[test]
    fn module_log_backup_documents_binary_transport_and_integrity_headers() {
        let value: serde_json::Value = serde_json::from_str(&normalized_json().unwrap()).unwrap();
        let response =
            &value["paths"]["/api/system/status/module-logs/backup"]["post"]["responses"]["200"];
        assert_eq!(response["content"]["application/x-tar"]["schema"]["type"], "string");
        assert_eq!(response["content"]["application/x-tar"]["schema"]["format"], "binary");
        for header in
            ["content-disposition", "x-rustzen-archive-sha256", "x-rustzen-archive-file-count"]
        {
            assert!(response["headers"][header].is_object(), "missing {header}");
        }
        assert_eq!(
            response["headers"]["x-rustzen-archive-file-count"]["schema"]["type"],
            "integer"
        );
    }

    #[test]
    fn synthetic_access_policies_have_explicit_openapi_security_metadata() {
        let operation = |operation, access| {
            serde_json::to_value(operation_for(&RouteContract {
                method: axum::http::Method::GET,
                path: "/synthetic".to_owned(),
                operation,
                access,
            }))
            .unwrap()
        };
        let public = operation(OperationDescriptor::ContractPublic, RegisteredAccess::Public);
        assert_eq!(public["security"], serde_json::json!([]));
        assert!(public["responses"].get("401").is_none());
        assert!(public["responses"].get("204").is_some());

        let any = operation(
            OperationDescriptor::ContractAny,
            RegisteredAccess::Any(vec!["cap:a".to_owned(), "cap:b".to_owned()]),
        );
        assert_eq!(any["x-rustzen-authorization"]["mode"], "any");
        assert_eq!(
            any["x-rustzen-authorization"]["capabilities"],
            serde_json::json!(["cap:a", "cap:b"])
        );

        let all = operation(
            OperationDescriptor::ContractAll,
            RegisteredAccess::All(vec!["cap:a".to_owned(), "cap:b".to_owned()]),
        );
        assert_eq!(all["x-rustzen-authorization"]["mode"], "all");
        assert_eq!(
            all["x-rustzen-authorization"]["capabilities"],
            serde_json::json!(["cap:a", "cap:b"])
        );
    }

    #[test]
    fn production_operations_expose_grounded_inputs_outputs_and_csv_transport() {
        let value: serde_json::Value = serde_json::from_str(&normalized_json().unwrap()).unwrap();
        let users = &value["paths"]["/api/system/users"]["get"];
        assert_eq!(users["parameters"][0]["name"], "current");
        assert_eq!(
            users["responses"]["200"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/ApiResponseUserItemRespList"
        );
        let avatar = &value["paths"]["/api/account/avatar"]["post"];
        assert_eq!(
            avatar["requestBody"]["content"]["multipart/form-data"]["schema"]["$ref"],
            "#/components/schemas/AvatarUpload"
        );
        let deployment_upload = &value["components"]["schemas"]["DeploymentUpload"];
        assert_eq!(
            deployment_upload["properties"]["component"]["$ref"],
            "#/components/schemas/DeployComponent"
        );
        assert_eq!(
            value["components"]["schemas"]["DeployComponent"]["enum"],
            serde_json::json!(["release"])
        );
        let csv = &value["paths"]["/api/manage/logs/export"]["get"];
        assert!(csv["responses"]["200"]["content"]["text/csv"].is_object());
        assert_eq!(csv["responses"]["200"]["content"]["text/csv"]["schema"]["type"], "string");
        let role_options = &value["paths"]["/api/system/roles/options"]["get"]["parameters"];
        assert_eq!(role_options[0]["name"], "q");
        assert_eq!(role_options[1]["name"], "limit");
        assert_eq!(
            value["paths"]["/api/manage/logs"]["get"]["responses"]["200"]["content"]["application/json"]
                ["schema"]["$ref"],
            "#/components/schemas/ApiResponseLogItemRespList"
        );
    }

    #[test]
    fn extractor_failures_are_grounded_for_every_query_path_and_multipart_route() {
        let document: serde_json::Value =
            serde_json::from_str(&normalized_json().unwrap()).unwrap();
        let assert_text_400 = |path: &str, method: &str| {
            let content = &document["paths"][path][method]["responses"]["400"]["content"];
            assert!(content["text/plain"].is_object(), "{method} {path} lacks text 400");
        };
        for (path, method) in [
            ("/api/manage/logs", "get"),
            ("/api/manage/logs/export", "get"),
            ("/api/manage/tasks/{task_key}/runs", "get"),
            ("/api/manage/deploy/list", "get"),
            ("/api/manage/deploy/cleanup", "post"),
            ("/api/system/menus", "get"),
            ("/api/system/menus/options", "get"),
            ("/api/system/roles", "get"),
            ("/api/system/roles/options", "get"),
            ("/api/system/users", "get"),
            ("/api/system/users/options", "get"),
        ] {
            assert_text_400(path, method);
        }
        for (path, method) in [
            ("/api/manage/deploy/{id}", "get"),
            ("/api/manage/deploy/{id}", "delete"),
            ("/api/manage/deploy/{id}/expire", "put"),
            ("/api/manage/deploy/{id}/deploy", "post"),
            ("/api/system/menus/{id}", "put"),
            ("/api/system/menus/{id}", "delete"),
            ("/api/system/roles/{id}", "put"),
            ("/api/system/roles/{id}", "delete"),
            ("/api/system/users/{id}", "put"),
            ("/api/system/users/{id}", "delete"),
            ("/api/system/users/{id}/password", "put"),
            ("/api/system/users/{id}/status", "put"),
        ] {
            assert_text_400(path, method);
        }
        for (path, method) in
            [("/api/account/avatar", "post"), ("/api/manage/deploy/upload", "post")]
        {
            let content = &document["paths"][path][method]["responses"]["413"]["content"];
            assert!(content["text/plain"].is_object(), "{method} {path} lacks text 413");
        }
        let login_400 =
            &document["paths"]["/api/auth/login"]["post"]["responses"]["400"]["content"];
        assert_eq!(
            login_400["application/json"]["schema"]["$ref"],
            "#/components/schemas/ApiErrorResponse"
        );
        assert!(login_400["text/plain"].is_object());
        assert_eq!(
            document["paths"]["/api/auth/login"]["post"]["responses"]["403"]["content"]["application/json"]
                ["schema"]["$ref"],
            "#/components/schemas/ApiErrorResponse"
        );
    }

    #[test]
    fn error_responses_are_typed_for_every_registered_operation() {
        let document: serde_json::Value =
            serde_json::from_str(&normalized_json().unwrap()).unwrap();
        for contract in crate::infra::app::documented_all_contracts() {
            let method = contract.method.as_str().to_ascii_lowercase();
            let operation = &document["paths"][&contract.path][&method];
            let responses = operation["responses"].as_object().expect("responses");
            for (status, response) in responses {
                if status.starts_with('2') {
                    continue;
                }
                let content = response["content"].as_object().unwrap_or_else(|| {
                    panic!("{status} {} has no response content", contract.operation.operation_id())
                });
                assert!(!content.is_empty(), "{status} response content is empty");
                if let Some(json) = content.get("application/json") {
                    assert_eq!(
                        json["schema"]["$ref"],
                        "#/components/schemas/ApiErrorResponse",
                        "{status} {} JSON error is not grounded",
                        contract.operation.operation_id()
                    );
                }
            }
            match contract.access {
                RegisteredAccess::Public => {
                    assert!(operation["security"].as_array().is_some());
                    if matches!(contract.operation, OperationDescriptor::Login) {
                        assert_eq!(
                            responses["401"]["content"]["application/json"]["schema"]["$ref"],
                            "#/components/schemas/ApiErrorResponse"
                        );
                        assert_eq!(
                            responses["403"]["content"]["application/json"]["schema"]["$ref"],
                            "#/components/schemas/ApiErrorResponse"
                        );
                    } else {
                        assert!(responses.get("401").is_none());
                        assert!(responses.get("403").is_none());
                    }
                }
                RegisteredAccess::Authenticated
                | RegisteredAccess::Require(_)
                | RegisteredAccess::Any(_)
                | RegisteredAccess::All(_) => {
                    assert_eq!(
                        responses["401"]["content"]["application/json"]["schema"]["$ref"],
                        "#/components/schemas/ApiErrorResponse"
                    );
                }
            }
            if matches!(
                contract.access,
                RegisteredAccess::Require(_) | RegisteredAccess::Any(_) | RegisteredAccess::All(_)
            ) {
                assert_eq!(
                    responses["403"]["content"]["application/json"]["schema"]["$ref"],
                    "#/components/schemas/ApiErrorResponse"
                );
            }
        }
    }
}
