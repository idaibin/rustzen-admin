use clap::Parser;
use serde_json::json;

use super::cli_output::Success;
use super::*;

#[test]
fn command_surface_uses_fixed_module_enum() {
    let cli = Cli::try_parse_from(["rz", "--json", "status", "reports"]).expect("valid CLI");
    assert!(cli.json);
    assert!(matches!(cli.command, Command::Status { module: Module::Reports }));
    assert!(Cli::try_parse_from(["rz", "status", "unknown"]).is_err());
}

#[test]
fn success_and_error_envelopes_are_stable() {
    let success = serde_json::to_value(Success {
        schema_version: SCHEMA_VERSION,
        ok: true,
        command: "version".to_string(),
        data: json!({"cli_version": "0.5.0"}),
    })
    .expect("success JSON");
    assert_eq!(success["schema_version"], 1);
    assert_eq!(success["ok"], true);
    assert_eq!(success["command"], "version");
    assert!(success.get("data").is_some());

    let failure = serde_json::to_value(Failure {
        schema_version: SCHEMA_VERSION,
        ok: false,
        command: "parse".to_string(),
        error: ErrorBody { code: "invalid_arguments", message: "invalid".to_string() },
    })
    .expect("failure JSON");
    assert_eq!(failure["ok"], false);
    assert_eq!(failure["error"]["code"], "invalid_arguments");
}

#[test]
fn config_reader_only_accepts_non_secret_endpoint_keys() {
    let directory = std::env::temp_dir().join(format!("rz-cli-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("temporary directory");
    let path = directory.join("rz.env");
    std::fs::write(
        &path,
        "RUSTZEN_INTERNAL_HOST=127.0.0.9\nRUSTZEN_ADMIN_PORT=19001\nRUSTZEN_JWT_SECRET=secret\nRUSTZEN_IPC_TOKEN=secret\n",
    )
    .expect("fixture config");
    let values = read_allowed_config(&path);
    assert_eq!(values.get("RUSTZEN_INTERNAL_HOST").map(String::as_str), Some("127.0.0.9"));
    assert_eq!(values.get("RUSTZEN_ADMIN_PORT").map(String::as_str), Some("19001"));
    assert!(!values.keys().any(|key| key.contains("SECRET") || key.contains("TOKEN")));
    std::fs::remove_file(path).expect("remove fixture");
    std::fs::remove_dir(directory).expect("remove temporary directory");
}

#[test]
fn endpoint_policy_rejects_non_loopback_reads() {
    assert!(is_loopback_host("127.0.0.1"));
    assert!(is_loopback_host("::1"));
    assert!(is_loopback_host("localhost"));
    assert!(!is_loopback_host("example.com"));
    assert!(!is_loopback_host("10.0.0.2"));
}
