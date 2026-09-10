use std::time::Duration;

use chromiumoxide::page::Page;
use serde_json::Value;
use uuid::Uuid;

use crate::{app::AppState, common::error::AppError};

use super::types::{Flow, Run, System};

mod artifacts;
mod dom;
mod layout;
mod session;
mod steps;
mod timeout;

pub(super) struct ExecutionContext<'a> {
    pub(super) state: &'a AppState,
    pub(super) run: &'a Run,
    pub(super) system: &'a System,
    pub(super) page: &'a Page,
    pub(super) input: &'a Value,
}

const BROWSER_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);
pub(super) const BROWSER_INIT_TIMEOUT: Duration = Duration::from_secs(30);
const VIEWPORT_WIDTH: u32 = 1440;
const VIEWPORT_HEIGHT: u32 = 900;

pub async fn execute(
    state: &AppState,
    run: &Run,
    flow: &Flow,
    system: &System,
    timeout: Duration,
    shutdown: tokio::sync::watch::Receiver<bool>,
) -> Result<(), AppError> {
    if *shutdown.borrow() {
        return Err(AppError::Interrupted);
    }
    // Each execution owns its profile; Chromium's shared default can be locked by
    // another run and can leak cookies between unrelated target systems.
    let profile = state.output_dir.join(&run.id).join(format!("browser-{}", Uuid::new_v4()));
    tokio::fs::create_dir_all(&profile).await?;
    let deadline = tokio::time::Instant::now() + timeout;
    let result =
        session::execute_with_profile(state, run, flow, system, &profile, deadline, shutdown).await;
    if let Err(error) = tokio::fs::remove_dir_all(&profile).await {
        tracing::warn!(run_id = run.id, %error, "Browser profile cleanup failed");
    }
    result
}

#[cfg(test)]
mod shutdown_tests {
    use super::*;

    #[tokio::test]
    async fn stopped_execution_never_launches_or_creates_a_profile() {
        let output_dir = std::env::temp_dir().join(format!("reports-stopped-{}", Uuid::new_v4()));
        let pool = sqlx::sqlite::SqlitePoolOptions::new().connect_lazy("sqlite::memory:").unwrap();
        let state = AppState {
            pool,
            output_dir: output_dir.clone(),
            browser_path: Some("/nonexistent-test-browser".into()),
            headless: true,
            max_concurrency: 1,
        };
        let run = Run {
            id: "run".into(),
            flow_id: "flow".into(),
            status: "running".into(),
            input_json: "{}".into(),
            error: None,
            created_at: String::new(),
            started_at: None,
            finished_at: None,
        };
        let flow = Flow {
            id: "flow".into(),
            system_id: "system".into(),
            name: String::new(),
            steps: vec![],
            created_at: String::new(),
            updated_at: String::new(),
        };
        let system = System {
            id: "system".into(),
            name: String::new(),
            base_url: "http://127.0.0.1".into(),
            enabled: true,
            notes: String::new(),
            created_at: String::new(),
            updated_at: String::new(),
        };
        let (_sender, receiver) = tokio::sync::watch::channel(true);
        assert!(matches!(
            execute(&state, &run, &flow, &system, Duration::from_secs(1), receiver).await,
            Err(AppError::Interrupted)
        ));
        assert!(!output_dir.exists());
    }
}
