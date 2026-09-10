use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

use super::run::Run;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScheduleCadence {
    Daily,
    Weekly,
}

impl ScheduleCadence {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Daily => "daily",
            Self::Weekly => "weekly",
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct ScheduleRow {
    pub id: String,
    pub flow_id: String,
    pub cadence: String,
    pub weekday: Option<i64>,
    pub due_time: String,
    pub input_json: String,
    pub description: String,
    pub enabled: bool,
    pub effective_at: String,
    pub revision: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveSchedule {
    pub flow_id: String,
    pub cadence: ScheduleCadence,
    pub weekday: Option<u8>,
    pub due_time: String,
    #[serde(default = "empty_object")]
    pub input: Value,
    #[serde(default)]
    pub description: String,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub id: String,
    pub flow_id: String,
    pub cadence: ScheduleCadence,
    pub weekday: Option<u8>,
    pub due_time: String,
    pub input: Value,
    pub description: String,
    pub enabled: bool,
    pub timezone: String,
    pub next_due: Option<String>,
    pub last_occurrence: Option<ScheduleOccurrence>,
    pub last_run: Option<Run>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleOccurrence {
    pub id: i64,
    pub schedule_id: String,
    pub occurrence_key: String,
    pub due_local: String,
    pub due_at: Option<String>,
    pub decided_at: String,
    pub decision: String,
    pub reason: Option<String>,
    pub run_id: Option<String>,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub run_id_snapshot: Option<String>,
}

fn empty_object() -> Value {
    Value::Object(Default::default())
}
