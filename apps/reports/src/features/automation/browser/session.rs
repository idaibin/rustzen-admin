use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::handler::viewport::Viewport;
use futures::{Future, StreamExt};
use serde_json::Value;

use crate::{app::AppState, common::error::AppError};

use super::super::types::{Flow, Run, System};
use super::{
    BROWSER_INIT_TIMEOUT, BROWSER_SHUTDOWN_TIMEOUT, ExecutionContext, VIEWPORT_HEIGHT,
    VIEWPORT_WIDTH, artifacts, execute_steps,
};

pub(super) async fn execute_with_profile(
    state: &AppState,
    run: &Run,
    flow: &Flow,
    system: &System,
    profile: &std::path::Path,
    deadline: tokio::time::Instant,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) -> Result<(), AppError> {
    let viewport = Viewport {
        width: VIEWPORT_WIDTH,
        height: VIEWPORT_HEIGHT,
        device_scale_factor: Some(1.0),
        emulating_mobile: false,
        is_landscape: true,
        has_touch: false,
    };
    let mut builder = BrowserConfig::builder()
        .user_data_dir(profile)
        .window_size(VIEWPORT_WIDTH, VIEWPORT_HEIGHT)
        .viewport(viewport)
        .launch_timeout(BROWSER_INIT_TIMEOUT);
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
    // chromiumoxide configures its child with kill_on_drop. A caller-level cap
    // also covers its CDP WebSocket connection, which is outside launch_timeout.
    let launch = launch_phase(Browser::launch(config));
    tokio::pin!(launch);
    let (mut browser, mut handler) = tokio::select! {
        _ = shutdown.wait_for(|stopped| *stopped) => return Err(AppError::Interrupted),
        result = &mut launch => result?,
    };
    let (handler_stopped_tx, mut handler_stopped_rx) = tokio::sync::oneshot::channel();
    let handle = tokio::spawn(async move {
        while let Some(result) = handler.next().await {
            if let Err(error) = result {
                tracing::error!(%error, "Browser CDP handler stopped");
                break;
            }
        }
        let _ = handler_stopped_tx.send(());
    });
    let work = tokio::time::timeout_at(deadline, async {
        let page = init_phase("new page", browser.new_page("about:blank"))
            .await?
            .map_err(AppError::internal)?;
        init_phase("initial target navigation", page.goto(&system.base_url))
            .await?
            .map_err(AppError::internal)?;
        let dimensions =
            page.evaluate("({ width: window.innerWidth, height: window.innerHeight })");
        let dimensions =
            init_phase("viewport evaluate", dimensions).await?.map_err(AppError::internal)?;
        let dimensions = dimensions.value().and_then(Value::as_object).ok_or_else(|| {
            AppError::internal(
                "browser initialization viewport validation failed: dimensions missing",
            )
        })?;
        if dimensions.get("width").and_then(Value::as_u64) != Some(u64::from(VIEWPORT_WIDTH))
            || dimensions.get("height").and_then(Value::as_u64) != Some(u64::from(VIEWPORT_HEIGHT))
        {
            return Err(AppError::internal(
                "browser initialization viewport validation failed: expected 1440x900",
            ));
        }
        let input: Value = serde_json::from_str(&run.input_json)?;
        let context = ExecutionContext { state, run, system, page: &page, input: &input };
        init_phase("initial live frame", artifacts::save_live_frame(state, &page, &run.id))
            .await??;
        let result = execute_steps(&context, flow).await;
        if result.is_err() {
            let _ = tokio::time::timeout(
                BROWSER_SHUTDOWN_TIMEOUT,
                artifacts::save_screenshot(state, &page, &run.id, "failure"),
            )
            .await;
        }
        result
    });
    let result = tokio::select! {
        biased;
        _ = shutdown.wait_for(|stopped| *stopped) => Err(AppError::Interrupted),
        _ = &mut handler_stopped_rx => Err(AppError::internal("browser CDP handler stopped")),
        result = work => result.unwrap_or(Err(AppError::TimedOut)),
    };
    close_browser(&mut browser, handle).await;
    result
}

async fn launch_phase<T, E>(future: impl Future<Output = Result<T, E>>) -> Result<T, AppError>
where
    E: std::fmt::Display,
{
    phase_timeout("launch", BROWSER_INIT_TIMEOUT, future).await?.map_err(|error| {
        AppError::internal(format!("browser initialization launch failed: {error}"))
    })
}

async fn init_phase<T>(
    phase: &'static str,
    future: impl Future<Output = T>,
) -> Result<T, AppError> {
    phase_timeout(phase, BROWSER_INIT_TIMEOUT, future).await
}

async fn phase_timeout<T>(
    phase: &'static str,
    timeout: std::time::Duration,
    future: impl Future<Output = T>,
) -> Result<T, AppError> {
    tokio::time::timeout(timeout, future)
        .await
        .map_err(|_| AppError::internal(format!("browser initialization timed out during {phase}")))
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

#[cfg(test)]
mod tests {
    use std::future;

    use super::phase_timeout;

    #[tokio::test]
    async fn launch_timeout_rejects_a_pending_complete_launch() {
        assert!(
            phase_timeout("launch", std::time::Duration::ZERO, future::pending::<()>())
                .await
                .is_err()
        );
    }
}
