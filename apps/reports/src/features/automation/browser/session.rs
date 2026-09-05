use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::handler::viewport::Viewport;
use futures::StreamExt;
use serde_json::Value;

use crate::{app::AppState, common::error::AppError};

use super::super::types::{Flow, Run, System};
use super::{
    BROWSER_SHUTDOWN_TIMEOUT, ExecutionContext, VIEWPORT_HEIGHT, VIEWPORT_WIDTH, artifacts,
    execute_steps,
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
        .viewport(viewport);
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
        let page = browser.new_page(&system.base_url).await.map_err(AppError::internal)?;
        let dimensions = page
            .evaluate("({ width: window.innerWidth, height: window.innerHeight })")
            .await
            .map_err(AppError::internal)?;
        let dimensions = dimensions
            .value()
            .and_then(Value::as_object)
            .ok_or_else(|| AppError::internal("browser did not report viewport dimensions"))?;
        if dimensions.get("width").and_then(Value::as_u64) != Some(u64::from(VIEWPORT_WIDTH))
            || dimensions.get("height").and_then(Value::as_u64) != Some(u64::from(VIEWPORT_HEIGHT))
        {
            return Err(AppError::internal("browser viewport differs from 1440x900"));
        }
        let input: Value = serde_json::from_str(&run.input_json)?;
        let context = ExecutionContext { state, run, system, page: &page, input: &input };
        artifacts::try_save_live_frame(state, &page, &run.id).await;
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
