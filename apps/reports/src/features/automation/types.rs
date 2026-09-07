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
    Goto {
        url: String,
    },
    Fill {
        selector: String,
        value: String,
    },
    Click {
        selector: String,
    },
    WaitFor {
        selector: String,
    },
    AssertText {
        selector: String,
        text: String,
    },
    AssertValue {
        selector: String,
        value: String,
    },
    AssertAbsent {
        selector: String,
    },
    Screenshot {
        name: Option<String>,
    },
    ScreenshotViewport {
        name: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    SetViewport {
        width: u32,
        height: u32,
    },
    #[serde(rename_all = "camelCase")]
    SetUiPreferences {
        theme: String,
        locale: String,
    },
    AssertNoHorizontalOverflow,
    #[serde(rename_all = "camelCase")]
    AssertElementLayout {
        selector: String,
        element_count: Option<u32>,
        visible_count: Option<u32>,
        max_height: Option<u32>,
        #[serde(default)]
        within_viewport_right: bool,
    },
    AssertFocus {
        selector: String,
    },
    #[serde(rename_all = "camelCase")]
    GuardExists {
        selector: String,
        on_missing: Option<String>,
    },
    PressKey {
        key: String,
    },
    #[serde(rename_all = "camelCase")]
    Pause {
        duration_ms: u64,
    },
}

impl FlowStep {
    pub fn action(&self) -> &'static str {
        match self {
            Self::Goto { .. } => "goto",
            Self::Fill { .. } => "fill",
            Self::Click { .. } => "click",
            Self::WaitFor { .. } => "waitFor",
            Self::AssertText { .. } => "assertText",
            Self::AssertValue { .. } => "assertValue",
            Self::AssertAbsent { .. } => "assertAbsent",
            Self::Screenshot { .. } => "screenshot",
            Self::ScreenshotViewport { .. } => "screenshotViewport",
            Self::SetViewport { .. } => "setViewport",
            Self::SetUiPreferences { .. } => "setUiPreferences",
            Self::AssertNoHorizontalOverflow => "assertNoHorizontalOverflow",
            Self::AssertElementLayout { .. } => "assertElementLayout",
            Self::AssertFocus { .. } => "assertFocus",
            Self::GuardExists { .. } => "guardExists",
            Self::PressKey { .. } => "pressKey",
            Self::Pause { .. } => "pause",
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

#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowOption {
    pub id: String,
    pub name: String,
    pub enabled: bool,
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallationSettings {
    pub timezone: String,
}
