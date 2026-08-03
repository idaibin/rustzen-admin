//! Programmatic OpenAPI output derived from Admin's registered route contracts.

use crate::{
    features::{auth::types::UserInfoResp, system::user::types::CreateUserRequest},
    infra::{
        app::documented_protected_routes,
        contract::{OperationDescriptor, RegisteredAccess},
    },
};
use utoipa::openapi::{
    ComponentsBuilder, Content, Info, OpenApi, OpenApiBuilder, Paths, Ref, ResponseBuilder,
    extensions::ExtensionsBuilder,
    path::{HttpMethod, Operation, OperationBuilder},
    request_body::RequestBodyBuilder,
    security::{Http, HttpAuthScheme, SecurityRequirement, SecurityScheme},
};

pub fn document() -> Result<OpenApi, crate::infra::contract::ContractError> {
    let (_, contracts) = documented_protected_routes();
    let mut paths = Paths::new();
    for contract in contracts {
        let operation = operation_for(&contract);
        let method = match contract.method {
            axum::http::Method::GET => HttpMethod::Get,
            axum::http::Method::POST => HttpMethod::Post,
            _ => unreachable!("the bounded pilot only registers GET and POST operations"),
        };
        paths.add_path_operation(&contract.path, vec![method], operation);
    }
    let mut components = ComponentsBuilder::new()
        .schema_from::<CreateUserRequest>()
        .schema_from::<UserInfoResp>()
        .build();
    components.schemas.insert(
        "ApiResponseI64".into(),
        envelope_schema(serde_json::json!({ "type": "integer", "format": "int64" })),
    );
    components.schemas.insert(
        "ApiResponseUserInfoResp".into(),
        envelope_schema(serde_json::json!({ "$ref": "#/components/schemas/UserInfoResp" })),
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
        OperationDescriptor::ContractPublic
        | OperationDescriptor::ContractAny
        | OperationDescriptor::ContractAll
        | OperationDescriptor::BenchmarkRequire => {
            operation.response("204", ResponseBuilder::new().description("No content").build())
        }
    };
    match &contract.access {
        RegisteredAccess::Public => {
            operation = operation.securities(Some(Vec::<SecurityRequirement>::new()))
        }
        RegisteredAccess::Authenticated => {
            operation = operation
                .security(SecurityRequirement::new("bearerAuth", Vec::<String>::new()))
                .response(
                    "401",
                    ResponseBuilder::new().description("Authentication required").build(),
                )
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
                .response(
                    "401",
                    ResponseBuilder::new().description("Authentication required").build(),
                )
                .response("403", ResponseBuilder::new().description("Missing capability").build())
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
    operation.build()
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
}
