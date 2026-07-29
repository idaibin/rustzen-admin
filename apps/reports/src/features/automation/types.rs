use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct System {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub enabled: bool,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveSystem {
    pub name: String,
    pub base_url: String,
    pub enabled: Option<bool>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub enum FlowStep {
    Goto { url: String },
    Fill { selector: String, value: String },
    Click { selector: String },
    WaitFor { selector: String },
    AssertText { selector: String, text: String },
    Screenshot { name: Option<String> },
}

impl FlowStep {
    pub fn action(&self) -> &'static str {
        match self {
            Self::Goto { .. } => "goto",
            Self::Fill { .. } => "fill",
            Self::Click { .. } => "click",
            Self::WaitFor { .. } => "waitFor",
            Self::AssertText { .. } => "assertText",
            Self::Screenshot { .. } => "screenshot",
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct FlowRow {
    pub id: String,
    pub system_id: String,
    pub name: String,
    pub steps_json: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Flow {
    pub id: String,
    pub system_id: String,
    pub name: String,
    pub steps: Vec<FlowStep>,
    pub created_at: String,
    pub updated_at: String,
}

impl TryFrom<FlowRow> for Flow {
    type Error = serde_json::Error;
    fn try_from(row: FlowRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            system_id: row.system_id,
            name: row.name,
            steps: serde_json::from_str(&row.steps_json)?,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveFlow {
    pub system_id: String,
    pub name: String,
    pub steps: Vec<FlowStep>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Run {
    pub id: String,
    pub flow_id: String,
    pub status: String,
    #[serde(skip_serializing)]
    pub input_json: String,
    pub error: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateRun {
    pub flow_id: String,
    #[serde(default = "empty_object")]
    pub input: Value,
}

fn empty_object() -> Value {
    Value::Object(Default::default())
}

#[derive(Debug, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunStep {
    pub id: i64,
    pub run_id: String,
    pub step_index: i64,
    pub action: String,
    pub status: String,
    pub duration_ms: Option<i64>,
    pub message: Option<String>,
    pub created_at: String,
}

#[derive(Debug, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
    pub id: String,
    pub run_id: String,
    pub kind: String,
    pub file_name: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub current: Option<i64>,
    pub page_size: Option<i64>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemFilter {
    pub system_id: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub run_retention_days: i64,
    pub artifact_retention_days: i64,
    pub default_step_timeout_seconds: i64,
    pub max_run_timeout_seconds: i64,
    pub updated_at: String,
}
