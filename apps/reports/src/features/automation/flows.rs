use chrono::Utc;
use rustzen_storage::SqlitePool;
use url::Url;
use uuid::Uuid;

use crate::common::error::AppError;

use super::{
    repo,
    systems::system,
    types::*,
    validation::{reject_sensitive_template, required},
};

pub async fn flows(pool: &SqlitePool, system_id: Option<&str>) -> Result<Vec<Flow>, AppError> {
    repo::flows(pool, system_id)
        .await?
        .into_iter()
        .map(Flow::try_from)
        .collect::<Result<_, _>>()
        .map_err(AppError::internal)
}

pub async fn flow_options(pool: &SqlitePool) -> Result<Vec<FlowOption>, AppError> {
    Ok(repo::flow_options(pool).await?)
}

pub async fn flow(pool: &SqlitePool, id: &str) -> Result<Flow, AppError> {
    repo::flow(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("flow not found".into()))?
        .try_into()
        .map_err(AppError::internal)
}

pub async fn create_flow(pool: &SqlitePool, input: SaveFlow) -> Result<Flow, AppError> {
    let system = system(pool, &input.system_id).await?;
    validate_flow(&system, &input.steps)?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    repo::insert_flow(
        pool,
        &id,
        &input.system_id,
        &required(input.name, "name", 100)?,
        &serde_json::to_string(&input.steps)?,
        &now,
    )
    .await?;
    flow(pool, &id).await
}

pub async fn update_flow(pool: &SqlitePool, id: &str, input: SaveFlow) -> Result<Flow, AppError> {
    let system = system(pool, &input.system_id).await?;
    validate_flow(&system, &input.steps)?;
    if !repo::update_flow(
        pool,
        id,
        &input.system_id,
        &required(input.name, "name", 100)?,
        &serde_json::to_string(&input.steps)?,
        &Utc::now().to_rfc3339(),
    )
    .await?
    {
        return Err(AppError::NotFound("flow not found".into()));
    }
    flow(pool, id).await
}

pub async fn delete_flow(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    match repo::delete_flow(pool, id).await {
        Ok(true) => Ok(()),
        Ok(false) => Err(AppError::NotFound("flow not found".into())),
        Err(sqlx::Error::Database(error)) if error.is_foreign_key_violation() => {
            Err(AppError::Conflict("flow is still referenced".into()))
        }
        Err(error) => Err(error.into()),
    }
}

fn validate_flow(system: &System, steps: &[FlowStep]) -> Result<(), AppError> {
    if steps.is_empty() || steps.len() > 100 {
        return Err(AppError::InvalidInput("flow must contain 1 to 100 steps".into()));
    }
    let base = Url::parse(&system.base_url).map_err(AppError::internal)?;
    for step in steps {
        match step {
            FlowStep::Goto { url } => {
                reject_sensitive_template(url)?;
                goto_target(&base, url)?;
            }
            FlowStep::Fill { selector, value } => {
                validate_selector(selector)?;
                reject_sensitive_template(value)?;
                if value.len() > 4000 {
                    return Err(AppError::InvalidInput("fill value is too long".into()));
                }
            }
            FlowStep::Click { selector } | FlowStep::WaitFor { selector } => {
                validate_selector(selector)?
            }
            FlowStep::AssertText { selector, text } => {
                validate_selector(selector)?;
                reject_sensitive_template(text)?;
                if text.len() > 1000 {
                    return Err(AppError::InvalidInput("asserted text is too long".into()));
                }
            }
            FlowStep::Screenshot { name } => {
                if name.as_ref().is_some_and(|value| value.len() > 100) {
                    return Err(AppError::InvalidInput("screenshot name is too long".into()));
                }
            }
            FlowStep::GuardExists { selector, on_missing } => {
                validate_selector(selector)?;
                if let Some(strategy) = on_missing {
                    match strategy.as_str() {
                        "continue" | "skipNext" | "stop" | "fail" | "error" => {}
                        _ => {
                            return Err(AppError::InvalidInput(format!(
                                "unsupported guardExists onMissing strategy: {strategy}"
                            )));
                        }
                    }
                }
            }
            FlowStep::PressKey { key } => {
                if key.trim().is_empty() || key.len() > 50 {
                    return Err(AppError::InvalidInput(
                        "pressKey must specify a key name with 1 to 50 characters".into(),
                    ));
                }
            }
            FlowStep::Pause { duration_ms } => {
                if *duration_ms > 300_000 {
                    return Err(AppError::InvalidInput(
                        "pause duration cannot exceed 300000ms".into(),
                    ));
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn goto_target(base: &Url, value: &str) -> Result<Url, AppError> {
    let target = base.join(value).map_err(|_| AppError::InvalidInput("invalid goto URL".into()))?;
    if target.origin() != base.origin() {
        return Err(AppError::InvalidInput("goto must remain on the system origin".into()));
    }
    Ok(target)
}

fn validate_selector(selector: &str) -> Result<(), AppError> {
    if selector.trim().is_empty() || selector.len() > 500 {
        return Err(AppError::InvalidInput("selector must contain 1 to 500 characters".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::{goto_target, validate_flow};
    use crate::features::automation::types::{FlowStep, System};

    #[test]
    fn substituted_goto_target_must_remain_on_the_system_origin() {
        let base = Url::parse("https://fixture.local").expect("base URL");
        assert!(goto_target(&base, "/relative").is_ok());
        assert!(goto_target(&base, "https://other.local/from-input").is_err());
    }

    #[test]
    fn validate_steps_accepts_guard_press_and_pause() {
        let system = System {
            id: "system".into(),
            name: "System".into(),
            base_url: "https://fixture.local".into(),
            enabled: true,
            notes: String::new(),
            created_at: String::new(),
            updated_at: String::new(),
        };
        let steps = vec![
            FlowStep::Goto { url: "/search".into() },
            FlowStep::GuardExists {
                selector: "#banner".into(),
                on_missing: Some("skipNext".into()),
            },
            FlowStep::Click { selector: "#banner-close".into() },
            FlowStep::Fill { selector: "#kw".into(), value: "test".into() },
            FlowStep::PressKey { key: "Enter".into() },
            FlowStep::Pause { duration_ms: 500 },
            FlowStep::Screenshot { name: Some("result".into()) },
        ];
        assert!(validate_flow(&system, &steps).is_ok());

        let invalid_guard = vec![FlowStep::GuardExists {
            selector: "#test".into(),
            on_missing: Some("invalid_strategy".into()),
        }];
        assert!(validate_flow(&system, &invalid_guard).is_err());
    }
}
