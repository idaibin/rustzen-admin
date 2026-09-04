use chromiumoxide::browser::{Browser, BrowserConfig};
use futures::StreamExt;
use serde_json::Value;

use crate::{app::AppState, common::error::AppError};

use super::super::types::{Flow, Run, System};
use super::{BROWSER_SHUTDOWN_TIMEOUT, ExecutionContext, artifacts, execute_steps};

pub(super) async fn execute_with_profile(
    state: &AppState,
    run: &Run,
    flow: &Flow,
    system: &System,
    profile: &std::path::Path,
    deadline: tokio::time::Instant,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) -> Result<(), AppError> {
    let mut builder = BrowserConfig::builder().user_data_dir(profile);
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
