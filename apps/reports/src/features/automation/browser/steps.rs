use std::time::{Duration, Instant};

use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use chrono::Utc;
use serde_json::Value;

use crate::common::error::AppError;

use super::{
    super::{
        repo, service,
        types::{Flow, FlowStep},
    },
    ExecutionContext, artifacts,
    dom::{assert_no_horizontal_overflow, fill_script, is_xpath, locate_element, wait_for},
    layout::assert_page_element_layout,
    timeout,
};

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
        let step_timeout = timeout::for_step(
            step,
            Duration::from_secs(settings.default_step_timeout_seconds as u64),
        );
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
            result = tokio::time::timeout(step_timeout, execute_step(context, step)) => result,
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
        artifacts::try_save_live_frame(context.state, context.page, &context.run.id).await;
        match outcome? {
            StepOutcome::Continue => {}
            StepOutcome::SkipNext => skip_next = true,
            StepOutcome::Stop => break,
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
            artifacts::save_screenshot(
                context.state,
                context.page,
                &context.run.id,
                name.as_deref().unwrap_or("screenshot"),
            )
            .await?;
            Ok(StepOutcome::Continue)
        }
        FlowStep::ScreenshotViewport { name } => {
            artifacts::save_viewport_screenshot(
                context.state,
                context.page,
                &context.run.id,
                name.as_deref().unwrap_or("screenshot"),
            )
            .await?;
            Ok(StepOutcome::Continue)
        }
        FlowStep::SetViewport { width, height } => {
            context
                .page
                .execute(SetDeviceMetricsOverrideParams::new(*width, *height, 1.0, *width == 390))
                .await
                .map_err(AppError::internal)?;
            Ok(StepOutcome::Continue)
        }
        FlowStep::SetUiPreferences { theme, locale } => {
            let theme = serde_json::to_string(theme)?;
            let locale = serde_json::to_string(locale)?;
            context.page.evaluate(format!("localStorage.setItem('rustzen-admin-theme', {theme}); localStorage.setItem('rustzen-admin-locale', {locale}); location.reload();")).await.map_err(AppError::internal)?;
            Ok(StepOutcome::Continue)
        }
        FlowStep::AssertNoHorizontalOverflow => {
            let metrics = context.page.evaluate("({ scrollWidth: document.documentElement.scrollWidth, innerWidth: window.innerWidth })").await.map_err(AppError::internal)?;
            assert_no_horizontal_overflow(metrics.value())?;
            Ok(StepOutcome::Continue)
        }
        FlowStep::AssertElementLayout {
            selector,
            element_count,
            visible_count,
            max_height,
            within_viewport_right,
            within_viewport,
        } => {
            assert_page_element_layout(
                context.page,
                selector,
                *element_count,
                *visible_count,
                *max_height,
                *within_viewport_right,
                *within_viewport,
            )
            .await?;
            Ok(StepOutcome::Continue)
        }
        FlowStep::AssertFocus { selector } => {
            let element = locate_element(context.page, selector).await?;
            let focused = element
                .call_js_fn("function() { return document.activeElement === this; }", false)
                .await
                .map_err(AppError::internal)?;
            if focused.result.value.as_ref().and_then(Value::as_bool) != Some(true) {
                return Err(AppError::Conflict("assertFocus did not match".into()));
            }
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
            tokio::time::sleep(Duration::from_millis(*duration_ms)).await;
            Ok(StepOutcome::Continue)
        }
    }
}
