use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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
        #[serde(default)]
        within_viewport: bool,
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
