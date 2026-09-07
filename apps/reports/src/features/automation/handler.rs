use axum::{
    body::Body,
    extract::{Extension, Path, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use rustzen_ipc::{DelegatedContext, ModuleJson, ModuleQuery, Page};
use std::path::{Path as FilePath, PathBuf};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::{
    app::AppState,
    common::api::{ApiResponse, AppResult},
    common::error::AppError,
};

use super::{repo, service, types::*};

pub async fn systems(State(state): State<AppState>) -> AppResult<Vec<System>> {
    Ok(ApiResponse::success(service::systems(&state.pool).await?))
}

pub async fn create_system(
    State(state): State<AppState>,
    ModuleJson(input): ModuleJson<SaveSystem>,
) -> AppResult<System> {
    Ok(ApiResponse::success(service::create_system(&state.pool, input).await?))
}

pub async fn update_system(
    State(state): State<AppState>,
    Path(id): Path<String>,
    ModuleJson(input): ModuleJson<SaveSystem>,
) -> AppResult<System> {
    Ok(ApiResponse::success(service::update_system(&state.pool, &id, input).await?))
}

pub async fn delete_system(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<()> {
    service::delete_system(&state.pool, &id).await?;
    Ok(ApiResponse::success(()))
}

pub async fn flows(
    State(state): State<AppState>,
    ModuleQuery(query): ModuleQuery<SystemFilter>,
) -> AppResult<Vec<Flow>> {
    Ok(ApiResponse::success(service::flows(&state.pool, query.system_id.as_deref()).await?))
}

pub async fn flow_options(State(state): State<AppState>) -> AppResult<Vec<FlowOption>> {
    Ok(ApiResponse::success(service::flow_options(&state.pool).await?))
}

pub async fn create_flow(
    State(state): State<AppState>,
    ModuleJson(input): ModuleJson<SaveFlow>,
) -> AppResult<Flow> {
    Ok(ApiResponse::success(service::create_flow(&state.pool, input).await?))
}

pub async fn update_flow(
    State(state): State<AppState>,
    Path(id): Path<String>,
    ModuleJson(input): ModuleJson<SaveFlow>,
) -> AppResult<Flow> {
    Ok(ApiResponse::success(service::update_flow(&state.pool, &id, input).await?))
}

pub async fn delete_flow(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<()> {
    service::delete_flow(&state.pool, &id).await?;
    Ok(ApiResponse::success(()))
}

pub async fn schedules(State(state): State<AppState>) -> AppResult<Vec<Schedule>> {
    Ok(ApiResponse::success(service::schedules(&state.pool).await?))
}

pub async fn settings() -> AppResult<InstallationSettings> {
    Ok(ApiResponse::success(InstallationSettings {
        timezone: crate::config::CONFIG.timezone().to_string(),
    }))
}

pub async fn create_schedule(
    State(state): State<AppState>,
    ModuleJson(input): ModuleJson<SaveSchedule>,
) -> AppResult<Schedule> {
    Ok(ApiResponse::success(service::create_schedule(&state.pool, input).await?))
}

pub async fn schedule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Schedule> {
    Ok(ApiResponse::success(service::schedule(&state.pool, &id).await?))
}

pub async fn update_schedule(
    State(state): State<AppState>,
    Path(id): Path<String>,
    ModuleJson(input): ModuleJson<SaveSchedule>,
) -> AppResult<Schedule> {
    Ok(ApiResponse::success(service::update_schedule(&state.pool, &id, input).await?))
}

pub async fn delete_schedule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<()> {
    service::delete_schedule(&state.pool, &id).await?;
    Ok(ApiResponse::success(()))
}

pub async fn runs(
    State(state): State<AppState>,
    ModuleQuery(query): ModuleQuery<ListQuery>,
) -> AppResult<Page<Run>> {
    Ok(ApiResponse::success(service::runs(&state.pool, query).await?))
}

pub async fn run(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Run> {
    Ok(ApiResponse::success(service::run(&state.pool, &id).await?))
}

pub async fn create_run(
    State(state): State<AppState>,
    Extension(context): Extension<DelegatedContext>,
    ModuleJson(input): ModuleJson<CreateRun>,
) -> AppResult<Run> {
    let initiator = context.user_id.ok_or(AppError::Internal)?;
    Ok(ApiResponse::success(service::create_run(&state.pool, input, initiator).await?))
}

pub async fn cancel_run(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Run> {
    Ok(ApiResponse::success(service::cancel_run(&state.pool, &id).await?))
}

pub async fn retry_run(
    State(state): State<AppState>,
    Extension(context): Extension<DelegatedContext>,
    Path(id): Path<String>,
) -> AppResult<Run> {
    let initiator = context.user_id.ok_or(AppError::Internal)?;
    Ok(ApiResponse::success(service::retry_run(&state.pool, &id, initiator).await?))
}

pub async fn run_steps(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Vec<RunStep>> {
    service::run(&state.pool, &id).await?;
    Ok(ApiResponse::success(repo::run_steps(&state.pool, &id).await?))
}

pub async fn run_artifacts(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Vec<Artifact>> {
    service::run(&state.pool, &id).await?;
    Ok(ApiResponse::success(repo::artifacts(&state.pool, &id).await?))
}

pub async fn artifact(
    State(state): State<AppState>,
    Path((run_id, artifact_id)): Path<(String, String)>,
) -> Result<Response, AppError> {
    let output_dir = run_output_dir(&state.output_dir, &run_id)?;
    let artifact = repo::artifact(&state.pool, &artifact_id, &run_id)
        .await?
        .ok_or_else(|| AppError::NotFound("artifact not found".into()))?;
    if FilePath::new(&artifact.file_name).file_name().and_then(|name| name.to_str())
        != Some(artifact.file_name.as_str())
    {
        return Err(AppError::Internal);
    }
    let file = tokio::fs::File::open(output_dir.join(&artifact.file_name)).await?;
    let mut response = Body::from_stream(ReaderStream::new(file)).into_response();
    response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("image/png"));
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", artifact.file_name))
            .map_err(AppError::internal)?,
    );
    Ok(response)
}

pub async fn live_frame(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let output_dir = run_output_dir(&state.output_dir, &id)?;
    service::run(&state.pool, &id).await?;
    let path = output_dir.join("live.png");
    let file = match tokio::fs::File::open(path).await {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(StatusCode::NO_CONTENT.into_response());
        }
        Err(error) => return Err(error.into()),
    };
    let mut response = Body::from_stream(ReaderStream::new(file)).into_response();
    response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("image/png"));
    response.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok(response)
}

fn run_output_dir(root: &FilePath, run_id: &str) -> Result<PathBuf, AppError> {
    let id =
        Uuid::parse_str(run_id).map_err(|_| AppError::InvalidInput("invalid run id".into()))?;
    if id.to_string() != run_id {
        return Err(AppError::InvalidInput("invalid run id".into()));
    }
    Ok(root.join(id.to_string()))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::run_output_dir;

    #[test]
    fn run_output_dir_accepts_only_canonical_uuid_components() {
        let id = "550e8400-e29b-41d4-a716-446655440000";
        assert_eq!(
            run_output_dir(Path::new("/tmp/reports"), id).unwrap(),
            Path::new("/tmp/reports").join(id)
        );
        assert!(run_output_dir(Path::new("/tmp/reports"), "../escape").is_err());
        assert!(
            run_output_dir(Path::new("/tmp/reports"), "550e8400e29b41d4a716446655440000").is_err()
        );
    }
}
