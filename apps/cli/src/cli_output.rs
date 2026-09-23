use serde::Serialize;
use serde_json::Value;

pub(super) const SCHEMA_VERSION: u8 = 1;

#[derive(Debug, Serialize)]
pub(super) struct Success<T: Serialize> {
    pub(super) schema_version: u8,
    pub(super) ok: bool,
    pub(super) command: String,
    pub(super) data: T,
}

#[derive(Debug, Serialize)]
pub(super) struct Failure {
    pub(super) schema_version: u8,
    pub(super) ok: bool,
    pub(super) command: String,
    pub(super) error: ErrorBody,
}

#[derive(Debug, Serialize)]
pub(super) struct ErrorBody {
    pub(super) code: &'static str,
    pub(super) message: String,
}

pub(super) fn emit(json_output: bool, command: &str, data: Value) {
    let output =
        Success { schema_version: SCHEMA_VERSION, ok: true, command: command.to_string(), data };
    if json_output {
        print_json(&output);
    } else {
        println!("{}", human_output(command, &output.data));
    }
}

pub(super) fn print_json(value: &impl Serialize) {
    match serde_json::to_string(value) {
        Ok(output) => println!("{output}"),
        Err(_) => {
            let fallback = r#"{"schema_version":1,"ok":false,"command":"serialize","error":{"code":"serialization_failed","message":"failed to serialize output"}}"#;
            println!("{fallback}");
        }
    }
}

fn human_output(command: &str, data: &Value) -> String {
    match command {
        "version" => format!(
            "rz {} (installed release: {})",
            data["cli_version"].as_str().unwrap_or("unknown"),
            data["installed_release"].as_str().unwrap_or("unavailable")
        ),
        _ => serde_json::to_string_pretty(data).unwrap_or_else(|_| "{}".to_string()),
    }
}
