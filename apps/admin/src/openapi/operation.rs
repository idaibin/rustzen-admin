use super::schema::*;
use super::*;

pub(super) fn operation_for(contract: &crate::infra::contract::RouteContract) -> Operation {
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
        | OperationDescriptor::GetDashboardModules
        | OperationDescriptor::ListNotifications
        | OperationDescriptor::GetNotificationUnreadCount
        | OperationDescriptor::GetNotification
        | OperationDescriptor::ReadNotification
        | OperationDescriptor::ReadAllNotifications => {
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
        let schema_type = if name == "id"
            && !matches!(
                contract.operation,
                OperationDescriptor::GetNotification | OperationDescriptor::ReadNotification
            ) {
            Type::Integer
        } else {
            Type::String
        };
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
        ListNotifications => vec![
            json_error("400", "Invalid inbox query or cursor"),
            json_error("403", "Account is disabled"),
            json_error("500", "Internal server error"),
        ],
        GetNotificationUnreadCount => vec![
            json_error("403", "Account is disabled"),
            json_error("500", "Internal server error"),
        ],
        GetNotification | ReadNotification => vec![
            json_error("403", "Account is disabled"),
            json_error("404", "Notification not found"),
            json_error("500", "Internal server error"),
        ],
        ReadAllNotifications => json_body_errors(&[
            json_error("403", "Account is disabled"),
            json_error("500", "Internal server error"),
        ]),
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
