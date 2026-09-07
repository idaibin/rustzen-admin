use super::*;
use crate::infra::contract::{RegisteredAccess, RouteContract};
#[test]
fn generated_document_matches_registered_paths_and_auth_metadata() {
    let value: serde_json::Value = serde_json::from_str(&normalized_json().unwrap()).unwrap();
    assert!(value["paths"]["/api/auth/me"]["get"]["security"].is_array());
    assert_eq!(
        value["paths"]["/api/system/users"]["post"]["x-rustzen-authorization"]["capabilities"][0],
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
    assert!(
        value["paths"]["/api/system/users"]["post"]["responses"]["422"]["content"]["text/plain"]
            .is_object()
    );
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
    assert_eq!(response["headers"]["x-rustzen-archive-file-count"]["schema"]["type"], "integer");
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
fn notification_contract_bounds_limit_uses_json_errors_and_hides_internal_sequence() {
    let document: serde_json::Value = serde_json::from_str(&normalized_json().unwrap()).unwrap();
    let operation = &document["paths"]["/api/notifications"]["get"];
    let limit = operation["parameters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|parameter| parameter["name"] == "limit")
        .expect("notification limit parameter");
    assert_eq!(limit["schema"]["minimum"], 1);
    assert_eq!(limit["schema"]["maximum"], 100);
    assert_eq!(
        operation["responses"]["400"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/ApiErrorResponse"
    );
    assert!(operation["responses"]["400"]["content"].get("text/plain").is_none());
    assert!(
        document["components"]["schemas"]["NotificationItem"]["properties"]
            .get("inboxSeq")
            .is_none()
    );
}

#[test]
fn extractor_failures_are_grounded_for_every_query_path_and_multipart_route() {
    let document: serde_json::Value = serde_json::from_str(&normalized_json().unwrap()).unwrap();
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
        ("/api/system/menus/inventory/{id}", "put"),
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
    for (path, method) in [("/api/account/avatar", "post"), ("/api/manage/deploy/upload", "post")] {
        let content = &document["paths"][path][method]["responses"]["413"]["content"];
        assert!(content["text/plain"].is_object(), "{method} {path} lacks text 413");
    }
    let login_400 = &document["paths"]["/api/auth/login"]["post"]["responses"]["400"]["content"];
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
    let document: serde_json::Value = serde_json::from_str(&normalized_json().unwrap()).unwrap();
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
