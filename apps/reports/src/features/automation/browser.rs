use std::time::{Duration, Instant};

use chromiumoxide::page::Page;
use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

use crate::{app::AppState, common::error::AppError};

use super::{
    repo, service,
    types::{Flow, FlowStep, Run, System},
};

mod artifacts;
mod session;

use artifacts::{save_screenshot, try_save_live_frame};

pub(super) struct ExecutionContext<'a> {
    pub(super) state: &'a AppState,
    pub(super) run: &'a Run,
    pub(super) system: &'a System,
    pub(super) page: &'a Page,
    pub(super) input: &'a Value,
}

const BROWSER_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);
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

enum StepOutcome {
    Continue,
    SkipNext,
    Stop,
}

pub(super) async fn execute_steps(
    context: &ExecutionContext<'_>,
    flow: &Flow,
) -> Result<(), AppError> {
    let settings = repo::settings(&context.state.pool).await?;
    let mut skip_next = false;
    for (index, step) in flow.steps.iter().enumerate() {
        if repo::run_cancel_requested(&context.state.pool, &context.run.id).await? {
            return Err(AppError::Cancelled);
        }
        if skip_next {
            skip_next = false;
            repo::insert_run_step(
                &context.state.pool,
                &context.run.id,
                index as i64,
                step.action(),
                "skipped",
                0,
                Some("skipped by guard condition"),
                &Utc::now().to_rfc3339(),
            )
            .await?;
            continue;
        }
        let started = Instant::now();
        let result = tokio::select! {
            cancellation = wait_for_cancellation(context) => {
                cancellation?;
                let duration = i64::try_from(started.elapsed().as_millis()).unwrap_or(i64::MAX);
                repo::insert_run_step(
                    &context.state.pool,
                    &context.run.id,
                    index as i64,
                    step.action(),
                    "cancelled",
                    duration,
                    Some("cancelled by user"),
                    &Utc::now().to_rfc3339(),
                )
                .await?;
                return Err(AppError::Cancelled);
            }
            result = tokio::time::timeout(
                Duration::from_secs(settings.default_step_timeout_seconds as u64),
                execute_step(context, step),
            ) => result,
        };
        let duration = i64::try_from(started.elapsed().as_millis()).unwrap_or(i64::MAX);
        let outcome = match result {
            Ok(result) => result,
            Err(_) => Err(AppError::Internal),
        };
        let (message, status) = match &outcome {
            Ok(_) => (None, "succeeded"),
            Err(error) => (Some(error.to_string()), "failed"),
        };
        repo::insert_run_step(
            &context.state.pool,
            &context.run.id,
            index as i64,
            step.action(),
            status,
            duration,
            message.as_deref(),
            &Utc::now().to_rfc3339(),
        )
        .await?;
        try_save_live_frame(context.state, context.page, &context.run.id).await;
        match outcome? {
            StepOutcome::Continue => {}
            StepOutcome::SkipNext => {
                skip_next = true;
            }
            StepOutcome::Stop => {
                break;
            }
        }
    }
    Ok(())
}

async fn wait_for_cancellation(context: &ExecutionContext<'_>) -> Result<(), AppError> {
    loop {
        if repo::run_cancel_requested(&context.state.pool, &context.run.id).await? {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn execute_step(
    context: &ExecutionContext<'_>,
    step: &FlowStep,
) -> Result<StepOutcome, AppError> {
    match step {
        FlowStep::Goto { url } => {
            let value = service::substitute(url, context.input)?;
            let base = url::Url::parse(&context.system.base_url).map_err(AppError::internal)?;
            let target = service::goto_target(&base, &value)?;
            context.page.goto(target.as_str()).await.map_err(AppError::internal)?;
            Ok(StepOutcome::Continue)
        }
        FlowStep::Fill { selector, value } => {
            let element = locate_element(context.page, selector).await?;
            let value = service::substitute(value, context.input)?;
            element.call_js_fn(fill_script(&value)?, false).await.map_err(AppError::internal)?;
            Ok(StepOutcome::Continue)
        }
        FlowStep::Click { selector } => {
            locate_element(context.page, selector)
                .await?
                .click()
                .await
                .map_err(AppError::internal)?;
            Ok(StepOutcome::Continue)
        }
        FlowStep::WaitFor { selector } => {
            wait_for(context.page, selector).await?;
            Ok(StepOutcome::Continue)
        }
        FlowStep::AssertText { selector, text } => {
            let actual = locate_element(context.page, selector)
                .await?
                .inner_text()
                .await
                .map_err(AppError::internal)?;
            let expected = service::substitute(text, context.input)?;
            if !actual.is_some_and(|actual| actual.contains(&expected)) {
                return Err(AppError::Conflict("assertText did not match".into()));
            }
            Ok(StepOutcome::Continue)
        }
        FlowStep::AssertValue { selector, value } => {
            let actual = locate_element(context.page, selector)
                .await?
                .call_js_fn("function() { return this.value; }", false)
                .await
                .map_err(AppError::internal)?;
            let expected = service::substitute(value, context.input)?;
            if actual.result.value.as_ref().and_then(Value::as_str) != Some(expected.as_str()) {
                return Err(AppError::Conflict("assertValue did not match".into()));
            }
            Ok(StepOutcome::Continue)
        }
        FlowStep::AssertAbsent { selector } => {
            let elements = if is_xpath(selector) {
                context.page.find_xpaths(selector.strip_prefix("xpath=").unwrap_or(selector)).await
            } else {
                context.page.find_elements(selector).await
            }
            .map_err(AppError::internal)?;
            if !elements.is_empty() {
                return Err(AppError::Conflict("assertAbsent found an element".into()));
            }
            Ok(StepOutcome::Continue)
        }
        FlowStep::Screenshot { name } => {
            save_screenshot(
                context.state,
                context.page,
                &context.run.id,
                name.as_deref().unwrap_or("screenshot"),
            )
            .await?;
            Ok(StepOutcome::Continue)
        }
        FlowStep::GuardExists { selector, on_missing } => {
            let exists = locate_element(context.page, selector).await.is_ok();
            if exists {
                Ok(StepOutcome::Continue)
            } else {
                match on_missing.as_deref().unwrap_or("continue") {
                    "skipNext" => Ok(StepOutcome::SkipNext),
                    "stop" => Ok(StepOutcome::Stop),
                    "fail" | "error" => Err(AppError::Conflict(format!(
                        "guardExists: element '{selector}' not found"
                    ))),
                    _ => Ok(StepOutcome::Continue),
                }
            }
        }
        FlowStep::PressKey { key } => {
            let encoded_key = serde_json::to_string(key)?;
            let script = format!(
                "(() => {{
                    const target = document.activeElement || document.body;
                    const init = {{ key: {encoded_key}, code: {encoded_key}, bubbles: true, cancelable: true }};
                    target.dispatchEvent(new KeyboardEvent('keydown', init));
                    target.dispatchEvent(new KeyboardEvent('keypress', init));
                    target.dispatchEvent(new KeyboardEvent('keyup', init));
                    if ({encoded_key} === 'Enter' && target.form) {{
                        target.form.dispatchEvent(new Event('submit', {{ bubbles: true, cancelable: true }}));
                    }}
                }})()"
            );
            context.page.evaluate(script).await.map_err(AppError::internal)?;
            Ok(StepOutcome::Continue)
        }
        FlowStep::Pause { duration_ms } => {
            let duration = Duration::from_millis((*duration_ms).min(30_000));
            tokio::time::sleep(duration).await;
            Ok(StepOutcome::Continue)
        }
    }
}

fn fill_script(value: &str) -> Result<String, serde_json::Error> {
    let encoded = serde_json::to_string(value)?;
    Ok(format!(
        "function() {{ const prototype = this instanceof HTMLInputElement ? HTMLInputElement.prototype : this instanceof HTMLTextAreaElement ? HTMLTextAreaElement.prototype : this instanceof HTMLSelectElement ? HTMLSelectElement.prototype : null; const setter = prototype && Object.getOwnPropertyDescriptor(prototype, 'value')?.set; if (!setter) throw new Error('element has no native value setter'); setter.call(this, {encoded}); this.dispatchEvent(new Event('input', {{ bubbles: true }})); this.dispatchEvent(new Event('change', {{ bubbles: true }})); }}"
    ))
}

fn is_xpath(selector: &str) -> bool {
    selector.starts_with("//") || selector.starts_with("xpath=")
}

async fn locate_element(page: &Page, selector: &str) -> Result<chromiumoxide::Element, AppError> {
    if is_xpath(selector) {
        let clean = selector.strip_prefix("xpath=").unwrap_or(selector);
        page.find_xpath(clean).await.map_err(AppError::internal)
    } else {
        page.find_element(selector).await.map_err(AppError::internal)
    }
}

async fn wait_for(page: &Page, selector: &str) -> Result<(), AppError> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if locate_element(page, selector).await.is_ok() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(AppError::Conflict("waitFor selector timed out".into()));
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
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

    #[test]
    fn xpath_selector_detection_supports_slash_and_prefix() {
        assert!(is_xpath("//button[@id='su']"));
        assert!(is_xpath("//*[@id='kw']"));
        assert!(is_xpath("xpath=//input"));
        assert!(is_xpath("xpath=//*[@class='title']"));
        assert!(!is_xpath("#kw"));
        assert!(!is_xpath("button.submit"));
        assert!(!is_xpath("[data-testid='btn']"));
    }
}

#[cfg(test)]
mod fill_tests {
    #[test]
    fn fill_uses_the_native_value_setter_and_json_escapes_input() {
        let script = super::fill_script("quoted \"value\"\nnext").expect("fill script");
        assert!(script.contains("this instanceof HTMLInputElement"));
        assert!(script.contains("this instanceof HTMLTextAreaElement"));
        assert!(script.contains("this instanceof HTMLSelectElement"));
        assert!(script.contains("Object.getOwnPropertyDescriptor(prototype, 'value')?.set"));
        assert!(script.contains("setter.call(this, \"quoted \\\"value\\\"\\nnext\")"));
        assert!(script.contains("new Event('input', { bubbles: true })"));
        assert!(script.contains("new Event('change', { bubbles: true })"));
    }
}
