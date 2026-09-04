use chrono::{NaiveTime, Utc};
use rustzen_ipc::{Page, Pagination};
use rustzen_storage::SqlitePool;
use uuid::Uuid;

use crate::common::error::AppError;

use super::{repo, types::*, validation::reject_sensitive_input};

pub(crate) use super::flows::goto_target;
pub use super::{
    flows::{create_flow, delete_flow, flow, flow_options, flows, update_flow},
    systems::{create_system, delete_system, system, systems, update_system},
    validation::substitute,
};

pub async fn schedules(pool: &SqlitePool) -> Result<Vec<Schedule>, AppError> {
    let mut schedules = Vec::new();
    for row in repo::schedules(pool).await? {
        schedules.push(schedule_from_row(pool, row).await?);
    }
    Ok(schedules)
}

pub async fn schedule(pool: &SqlitePool, id: &str) -> Result<Schedule, AppError> {
    let row = repo::schedule(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("schedule not found".into()))?;
    schedule_from_row(pool, row).await
}

pub async fn create_schedule(pool: &SqlitePool, input: SaveSchedule) -> Result<Schedule, AppError> {
    let validated = validate_schedule(pool, input).await?;
    let id = Uuid::new_v4().to_string();
    repo::insert_schedule(
        pool,
        &id,
        &validated.flow_id,
        validated.cadence.as_str(),
        validated.weekday,
        &validated.due_time,
        &validated.input_json,
        &validated.description,
        validated.enabled,
        &validated.now,
    )
    .await?;
    schedule(pool, &id).await
}

pub async fn update_schedule(
    pool: &SqlitePool,
    id: &str,
    input: SaveSchedule,
) -> Result<Schedule, AppError> {
    let current = repo::schedule(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("schedule not found".into()))?;
    let keep_enabled = input.enabled.is_none();
    let validated = validate_schedule(pool, input).await?;
    let enabled = if keep_enabled { current.enabled } else { validated.enabled };
    if !repo::update_schedule(
        pool,
        id,
        &validated.flow_id,
        validated.cadence.as_str(),
        validated.weekday,
        &validated.due_time,
        &validated.input_json,
        &validated.description,
        enabled,
        &validated.now,
    )
    .await?
    {
        return Err(AppError::NotFound("schedule not found".into()));
    }
    schedule(pool, id).await
}

pub async fn delete_schedule(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    if repo::delete_schedule(pool, id).await? {
        Ok(())
    } else {
        Err(AppError::NotFound("schedule not found".into()))
    }
}

struct ValidatedSchedule {
    flow_id: String,
    cadence: ScheduleCadence,
    weekday: Option<u8>,
    due_time: String,
    input_json: String,
    description: String,
    enabled: bool,
    now: String,
}

async fn validate_schedule(
    pool: &SqlitePool,
    input: SaveSchedule,
) -> Result<ValidatedSchedule, AppError> {
    let flow = flow(pool, &input.flow_id).await?;
    let system = system(pool, &flow.system_id).await?;
    if !system.enabled {
        return Err(AppError::InvalidInput("schedule target system must be enabled".into()));
    }
    if !input.input.is_object() {
        return Err(AppError::InvalidInput("input must be an object".into()));
    }
    reject_sensitive_input(&input.input)?;
    let due_time = NaiveTime::parse_from_str(input.due_time.trim(), "%H:%M")
        .map_err(|_| AppError::InvalidInput("dueTime must use HH:MM (24-hour) format".into()))?
        .format("%H:%M")
        .to_string();
    let weekday = match input.cadence {
        ScheduleCadence::Daily => {
            if input.weekday.is_some() {
                return Err(AppError::InvalidInput("daily schedule must not set weekday".into()));
            }
            None
        }
        ScheduleCadence::Weekly => {
            let weekday = input
                .weekday
                .ok_or_else(|| AppError::InvalidInput("weekly schedule requires weekday".into()))?;
            if weekday > 6 {
                return Err(AppError::InvalidInput("weekday must be between 0 and 6".into()));
            }
            Some(weekday)
        }
    };
    Ok(ValidatedSchedule {
        flow_id: input.flow_id,
        cadence: input.cadence,
        weekday,
        due_time,
        input_json: serde_json::to_string(&input.input)?,
        description: input.description.trim().chars().take(1000).collect(),
        enabled: input.enabled.unwrap_or(true),
        now: Utc::now().to_rfc3339(),
    })
}

async fn schedule_from_row(pool: &SqlitePool, row: ScheduleRow) -> Result<Schedule, AppError> {
    let cadence = match row.cadence.as_str() {
        "daily" => ScheduleCadence::Daily,
        "weekly" => ScheduleCadence::Weekly,
        _ => return Err(AppError::Internal),
    };
    let weekday =
        row.weekday.map(|value| u8::try_from(value).map_err(|_| AppError::Internal)).transpose()?;
    let input = serde_json::from_str(&row.input_json)?;
    let timezone = crate::config::CONFIG.timezone().to_string();
    let next_due = if row.enabled {
        super::scheduler::next_due_at(&row, Utc::now(), &timezone)?
    } else {
        None
    };
    Ok(Schedule {
        id: row.id.clone(),
        flow_id: row.flow_id.clone(),
        cadence,
        weekday,
        due_time: row.due_time,
        input,
        description: row.description,
        enabled: row.enabled,
        timezone,
        next_due,
        last_occurrence: repo::last_schedule_occurrence(pool, &row.id).await?,
        last_run: repo::last_schedule_run(pool, &row.id).await?,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

pub async fn create_run(pool: &SqlitePool, input: CreateRun) -> Result<Run, AppError> {
    flow(pool, &input.flow_id).await?;
    if !input.input.is_object() {
        return Err(AppError::InvalidInput("input must be an object".into()));
    }
    reject_sensitive_input(&input.input)?;
    let id = Uuid::new_v4().to_string();
    repo::insert_run(
        pool,
        &id,
        &input.flow_id,
        &serde_json::to_string(&input.input)?,
        &Utc::now().to_rfc3339(),
    )
    .await?;
    run(pool, &id).await
}

pub async fn runs(pool: &SqlitePool, query: ListQuery) -> Result<Page<Run>, AppError> {
    let page = Pagination::parse(query.current, query.page_size)
        .map_err(|_| AppError::InvalidInput("invalid pagination".into()))?;
    let (data, total) =
        repo::runs(pool, page.offset(), page.page_size(), query.status.as_deref()).await?;
    Ok(Page { data, total, success: true })
}
pub async fn run(pool: &SqlitePool, id: &str) -> Result<Run, AppError> {
    repo::run(pool, id).await?.ok_or_else(|| AppError::NotFound("run not found".into()))
}
pub async fn cancel_run(pool: &SqlitePool, id: &str) -> Result<Run, AppError> {
    if !repo::cancel_run(pool, id, &Utc::now().to_rfc3339()).await? {
        return Err(AppError::Conflict("only queued or running runs can be cancelled".into()));
    }
    run(pool, id).await
}
pub async fn retry_run(pool: &SqlitePool, id: &str) -> Result<Run, AppError> {
    super::retry::retry_run(pool, id).await
}
