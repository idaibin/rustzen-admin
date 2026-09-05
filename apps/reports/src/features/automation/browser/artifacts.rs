use chromiumoxide::{
    cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams,
    cdp::browser_protocol::page::CaptureScreenshotFormat,
    page::{Page, ScreenshotParams},
};
use chrono::Utc;
use uuid::Uuid;

use crate::{app::AppState, common::error::AppError};

use super::{BROWSER_SHUTDOWN_TIMEOUT, repo};

const MAX_SCREENSHOT_PIXELS: u64 = 4_000_000;
const MAX_SCREENSHOT_BYTES: usize = 4 * 1024 * 1024;
const MAX_RUN_SCREENSHOT_BYTES: u64 = 16 * 1024 * 1024;

pub(super) async fn try_save_live_frame(state: &AppState, page: &Page, run_id: &str) {
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

pub(super) async fn save_live_frame(
    state: &AppState,
    page: &Page,
    run_id: &str,
) -> Result<(), AppError> {
    let artifact_id = format!("{run_id}-live");
    let file_name = "live.png";
    let dir = state.output_dir.join(run_id);
    tokio::fs::create_dir_all(&dir).await?;
    let bytes = page
        .screenshot(ScreenshotParams::builder().format(CaptureScreenshotFormat::Png).build())
        .await
        .map_err(AppError::internal)?;
    validate_screenshot(&bytes)?;
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

pub(super) async fn save_screenshot(
    state: &AppState,
    page: &Page,
    run_id: &str,
    name: &str,
) -> Result<(), AppError> {
    save_named_screenshot(state, page, run_id, name, true).await
}

pub(super) async fn save_viewport_screenshot(
    state: &AppState,
    page: &Page,
    run_id: &str,
    name: &str,
) -> Result<(), AppError> {
    save_named_screenshot(state, page, run_id, name, false).await
}

async fn save_named_screenshot(
    state: &AppState,
    page: &Page,
    run_id: &str,
    name: &str,
    full_page: bool,
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
    if full_page {
        validate_full_page_layout(page).await?;
    }
    let viewport = if full_page { Some(current_viewport(page).await?) } else { None };
    let capture = page
        .screenshot(
            ScreenshotParams::builder()
                .format(CaptureScreenshotFormat::Png)
                .full_page(full_page)
                .build(),
        )
        .await;
    // chromiumoxide clears its emulated metrics after a full-page capture.
    // Restore the session contract before a later step can navigate or inspect
    // a responsive page.
    let restored = if let Some((width, height, scale, mobile)) = viewport {
        Some(page.execute(SetDeviceMetricsOverrideParams::new(width, height, scale, mobile)).await)
    } else {
        None
    };
    let bytes = capture.map_err(AppError::internal)?;
    if let Some(restored) = restored {
        restored.map_err(AppError::internal)?;
    }
    validate_screenshot(&bytes)?;
    if screenshot_bytes_in_dir(&dir).await? + u64::try_from(bytes.len()).unwrap_or(u64::MAX)
        > MAX_RUN_SCREENSHOT_BYTES
    {
        return Err(AppError::Conflict("run screenshot quota exceeded".into()));
    }
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

async fn current_viewport(page: &Page) -> Result<(u32, u32, f64, bool), AppError> {
    let metrics = page
        .evaluate("({ width: window.innerWidth, height: window.innerHeight, scale: window.devicePixelRatio })")
        .await
        .map_err(AppError::internal)?;
    let metrics = metrics
        .value()
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| AppError::internal("browser did not report viewport metrics"))?;
    let width = metrics
        .get("width")
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| *value > 0)
        .ok_or_else(|| AppError::internal("browser viewport width is invalid"))?;
    let height = metrics
        .get("height")
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| *value > 0)
        .ok_or_else(|| AppError::internal("browser viewport height is invalid"))?;
    let scale = metrics
        .get("scale")
        .and_then(serde_json::Value::as_f64)
        .filter(|value| value.is_finite() && *value > 0.)
        .ok_or_else(|| AppError::internal("browser viewport scale is invalid"))?;
    Ok((width, height, scale, width == 390))
}

fn validate_screenshot(bytes: &[u8]) -> Result<(), AppError> {
    if bytes.len() > MAX_SCREENSHOT_BYTES {
        return Err(AppError::Conflict("screenshot exceeds byte limit".into()));
    }
    let Some(header) = bytes.get(0..24) else {
        return Err(AppError::Conflict("invalid PNG screenshot".into()));
    };
    if header[..8] != [137, 80, 78, 71, 13, 10, 26, 10]
        || header[8..12] != [0, 0, 0, 13]
        || header[12..16] != *b"IHDR"
    {
        return Err(AppError::Conflict("invalid PNG screenshot".into()));
    }
    let width = u32::from_be_bytes(header[16..20].try_into().expect("PNG width"));
    let height = u32::from_be_bytes(header[20..24].try_into().expect("PNG height"));
    if width == 0
        || height == 0
        || u64::from(width)
            .checked_mul(u64::from(height))
            .is_none_or(|pixels| pixels > MAX_SCREENSHOT_PIXELS)
    {
        return Err(AppError::Conflict("screenshot exceeds pixel limit".into()));
    }
    Ok(())
}

async fn validate_full_page_layout(page: &Page) -> Result<(), AppError> {
    let size = page.layout_metrics().await.map_err(AppError::internal)?.css_content_size;
    let width = size.width.ceil();
    let height = size.height.ceil();
    if !width.is_finite()
        || !height.is_finite()
        || width <= 0.
        || height <= 0.
        || width > u32::MAX as f64
        || height > u32::MAX as f64
        || (width as u64)
            .checked_mul(height as u64)
            .is_none_or(|pixels| pixels > MAX_SCREENSHOT_PIXELS)
    {
        return Err(AppError::Conflict("screenshot exceeds pixel limit".into()));
    }
    Ok(())
}

async fn screenshot_bytes_in_dir(dir: &std::path::Path) -> Result<u64, AppError> {
    let mut entries = tokio::fs::read_dir(dir).await?;
    let mut total = 0_u64;
    while let Some(entry) = entries.next_entry().await? {
        if entry.path().extension().is_some_and(|extension| extension == "png") {
            total = total.saturating_add(entry.metadata().await?.len());
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::{MAX_SCREENSHOT_BYTES, validate_screenshot};

    #[test]
    fn screenshot_bounds_reject_large_dimensions_and_bytes() {
        let mut oversized_dimensions = vec![0; 24];
        oversized_dimensions[..8].copy_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);
        oversized_dimensions[8..12].copy_from_slice(&13_u32.to_be_bytes());
        oversized_dimensions[12..16].copy_from_slice(b"IHDR");
        oversized_dimensions[16..20].copy_from_slice(&2_001_u32.to_be_bytes());
        oversized_dimensions[20..24].copy_from_slice(&2_000_u32.to_be_bytes());
        assert!(validate_screenshot(&oversized_dimensions).is_err());
        let mut zero = oversized_dimensions.clone();
        zero[16..20].copy_from_slice(&0_u32.to_be_bytes());
        assert!(validate_screenshot(&zero).is_err());
        let mut invalid_signature = oversized_dimensions.clone();
        invalid_signature[0] = 0;
        assert!(validate_screenshot(&invalid_signature).is_err());
        assert!(validate_screenshot(&vec![0; MAX_SCREENSHOT_BYTES + 1]).is_err());
    }
}
