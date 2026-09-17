//! Programmatic OpenAPI output derived from Admin's registered route contracts.

#[cfg(feature = "notifications")]
use crate::features::notifications::types::{
    InboxListResponse, NotificationItem, ReadAllRequest, ReadAllResponse, ReadResponse,
    UnreadCountResponse,
};
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
                MemoryResourceStatus, ModuleDatabaseStatus, SqliteStorageStatus,
                SystemStatusOverview, SystemStorageStatus,
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
        .schema_from::<ModuleDatabaseStatus>()
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
    #[cfg(feature = "notifications")]
    {
        macro_rules! insert_schema {
            ($type:ty) => {
                components.schemas.insert(
                    <$type as utoipa::ToSchema>::name().into_owned(),
                    <$type as utoipa::PartialSchema>::schema(),
                );
            };
        }
        insert_schema!(NotificationItem);
        insert_schema!(InboxListResponse);
        insert_schema!(UnreadCountResponse);
        insert_schema!(ReadResponse);
        insert_schema!(ReadAllRequest);
        insert_schema!(ReadAllResponse);
    }
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
        .info(Info::new("Rustzen Admin contract trial", env!("CARGO_PKG_VERSION")))
        .paths(paths)
        .components(Some(components))
        .build())
}

mod operation;
mod schema;

use operation::operation_for;
use schema::{envelope_schema, error_schema, multipart_schema, response_schemas};

pub fn normalized_json() -> Result<String, Box<dyn std::error::Error>> {
    Ok(serde_json::to_string_pretty(&document()?)?)
}

#[cfg(test)]
mod tests;
