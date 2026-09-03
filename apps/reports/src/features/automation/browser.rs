use std::time::{Duration, Instant};

use chromiumoxide::{
    browser::{Browser, BrowserConfig},
    cdp::browser_protocol::page::CaptureScreenshotFormat,
    page::{Page, ScreenshotParams},
};
use chrono::Utc;
use futures::StreamExt;
use serde_json::Value;
use uuid::Uuid;

use crate::{app::AppState, common::error::AppError};

use super::{
    repo, service,
    types::{Flow, FlowStep, Run, System},
};

struct ExecutionContext<'a> {
    state: &'a AppState,
    run: &'a Run,
    system: &'a System,
    page: &'a Page,
    input: &'a Value,
}

const BROWSER_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

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
    let result = execute_with_profile(state, run, flow, system, &profile, deadline, shutdown).await;
    if let Err(error) = tokio::fs::remove_dir_all(&profile).await {
        tracing::warn!(run_id = run.id, %error, "Browser profile cleanup failed");
    }
    result
}

async fn execute_with_profile(
    state: &AppState,
    run: &Run,
    flow: &Flow,
    system: &System,
    profile: &std::path::Path,
    deadline: tokio::time::Instant,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) -> Result<(), AppError> {
    let mut builder = BrowserConfig::builder()
        .user_data_dir(profile)
        .no_sandbox();
    if let Some(path) = state.browser_path.as_deref() {
        builder = builder.chrome_executable(path);
    }
    if !state.headless {
        builder = builder.with_head();
    }
    let config = builder.build().map_err(AppError::internal)?;
    if *shutdown.borrow() {
        return Err(AppError::Interrupted);
    }
    let (mut browser, mut handler) = tokio::time::timeout_at(deadline, Browser::launch(config))
        .await
        .map_err(|_| AppError::TimedOut)?
        .map_err(AppError::internal)?;
    let handle = tokio::spawn(async move {
        while let Some(result) = handler.next().await {
            if result.is_err() {
                break;
            }
        }
    });
    let work = tokio::time::timeout_at(deadline, async {
        let page = browser.new_page(&system.base_url).await.map_err(AppError::internal)?;
        let input: Value = serde_json::from_str(&run.input_json)?;
        let context = ExecutionContext { state, run, system, page: &page, input: &input };
        try_save_live_frame(state, &page, &run.id).await;
        let result = execute_steps(&context, flow).await;
        if result.is_err() {
            let _ = tokio::time::timeout(
                BROWSER_SHUTDOWN_TIMEOUT,
                save_screenshot(state, &page, &run.id, "failure"),
            )
            .await;
        }
        result
    });
    let result = tokio::select! {
        biased;
        _ = shutdown.wait_for(|stopped| *stopped) => Err(AppError::Interrupted),
        result = work => result.unwrap_or(Err(AppError::TimedOut)),
    };
    close_browser(&mut browser, handle).await;
    result
}

async fn close_browser(browser: &mut Browser, mut handler: tokio::task::JoinHandle<()>) {
    let closed =
        matches!(tokio::time::timeout(BROWSER_SHUTDOWN_TIMEOUT, browser.close()).await, Ok(Ok(_)));
    let exited = closed
        && matches!(
            tokio::time::timeout(BROWSER_SHUTDOWN_TIMEOUT, browser.wait()).await,
            Ok(Ok(_))
        );
    if !exited {
        tracing::warn!("Browser close did not complete; forcing process shutdown");
        let _ = tokio::time::timeout(BROWSER_SHUTDOWN_TIMEOUT, browser.kill()).await;
    }

    if tokio::time::timeout(BROWSER_SHUTDOWN_TIMEOUT, &mut handler).await.is_err() {
        tracing::warn!("Browser handler did not stop; aborting handler task");
        handler.abort();
        let _ = handler.await;
    }
}

enum StepOutcome {
    Continue,
    SkipNext,
    Stop,
}

async fn execute_steps(context: &ExecutionContext<'_>, flow: &Flow) -> Result<(), AppError> {
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

async fn try_save_live_frame(state: &AppState, page: &Page, run_id: &str) {
    match tokio::time::timeout(BROWSER_SHUTDOWN_TIMEOUT, save_live_frame(state, page, run_id)).await
    {
        Ok(Ok(())) => {}
        Ok(Err(error)) => {
            tracing::warn!(run_id, %error, "Live frame capture failed");
        }
        Err(_) => {
            tracing::warn!(run_id, "Live frame capture timed out");
        }
    }
}

async fn save_live_frame(state: &AppState, page: &Page, run_id: &str) -> Result<(), AppError> {
    let artifact_id = format!("{run_id}-live");
    let file_name = "live.png";
    let dir = state.output_dir.join(run_id);
    tokio::fs::create_dir_all(&dir).await?;
    let bytes = page
        .screenshot(ScreenshotParams::builder().format(CaptureScreenshotFormat::Png).build())
        .await
        .map_err(AppError::internal)?;
    let temporary = dir.join(format!("live-{}.tmp", Uuid::new_v4()));
    tokio::fs::write(&temporary, bytes).await?;
    if let Err(error) = tokio::fs::rename(&temporary, dir.join(file_name)).await {
        let _ = tokio::fs::remove_file(&temporary).await;
        return Err(error.into());
    }
    repo::upsert_live_artifact(
        &state.pool,
        &artifact_id,
        run_id,
        file_name,
        &Utc::now().to_rfc3339(),
    )
    .await?;
    Ok(())
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
            let encoded = serde_json::to_string(&value)?;
            element
                .call_js_fn(
                    format!(
                        "function() {{ this.value = {encoded}; this.dispatchEvent(new Event('input', {{ bubbles: true }})); this.dispatchEvent(new Event('change', {{ bubbles: true }})); }}"
                    ),
                    false,
                )
                .await
                .map_err(AppError::internal)?;
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

async fn save_screenshot(
    state: &AppState,
    page: &Page,
    run_id: &str,
    name: &str,
) -> Result<(), AppError> {
    let id = Uuid::new_v4().to_string();
    let safe: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(60)
        .collect();
    let file_name = format!("{}-{}.png", if safe.is_empty() { "screenshot" } else { &safe }, id);
    let dir = state.output_dir.join(run_id);
    tokio::fs::create_dir_all(&dir).await?;
    let bytes = page
        .screenshot(
            ScreenshotParams::builder()
                .format(CaptureScreenshotFormat::Png)
                .full_page(true)
                .build(),
        )
        .await
        .map_err(AppError::internal)?;
    tokio::fs::write(dir.join(&file_name), bytes).await?;
    repo::insert_artifact(
        &state.pool,
        &id,
        run_id,
        "screenshot",
        &file_name,
        &Utc::now().to_rfc3339(),
    )
    .await?;
    Ok(())
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
