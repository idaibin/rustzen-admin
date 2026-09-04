use chrono::{NaiveTime, Utc};
use rustzen_ipc::{Page, Pagination};
use rustzen_storage::SqlitePool;
use serde_json::Value;
use url::Url;
use uuid::Uuid;

use crate::common::error::AppError;

use super::{repo, types::*};

pub async fn systems(pool: &SqlitePool) -> Result<Vec<System>, AppError> {
    Ok(repo::systems(pool).await?)
}
pub async fn create_system(pool: &SqlitePool, input: SaveSystem) -> Result<System, AppError> {
    let (id, name, url, enabled, notes, now) = validated_system(input)?;
    repo::insert_system(pool, &id, &name, &url, enabled, &notes, &now).await?;
    system(pool, &id).await
}
pub async fn update_system(
    pool: &SqlitePool,
    id: &str,
    input: SaveSystem,
) -> Result<System, AppError> {
    let (_, name, url, enabled, notes, now) = validated_system(input)?;
    if !repo::update_system(pool, id, &name, &url, enabled, &notes, &now).await? {
        return Err(AppError::NotFound("system not found".into()));
    }
    system(pool, id).await
}
pub async fn system(pool: &SqlitePool, id: &str) -> Result<System, AppError> {
    repo::system(pool, id).await?.ok_or_else(|| AppError::NotFound("system not found".into()))
}
pub async fn delete_system(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    match repo::delete_system(pool, id).await {
        Ok(true) => Ok(()),
        Ok(false) => Err(AppError::NotFound("system not found".into())),
        Err(sqlx::Error::Database(error)) if error.is_foreign_key_violation() => {
            Err(AppError::Conflict("system is still referenced".into()))
        }
        Err(error) => Err(error.into()),
    }
}

fn validated_system(
    input: SaveSystem,
) -> Result<(String, String, String, bool, String, String), AppError> {
    let name = required(input.name, "name", 100)?;
    let parsed = Url::parse(input.base_url.trim())
        .map_err(|_| AppError::InvalidInput("baseUrl must be an absolute HTTP/HTTPS URL".into()))?;
    if !matches!(parsed.scheme(), "http" | "https")
        || parsed.host_str().is_none()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        return Err(AppError::InvalidInput("baseUrl must be an HTTP/HTTPS origin".into()));
    }
    let origin = parsed.origin().ascii_serialization();
    Ok((
        Uuid::new_v4().to_string(),
        name,
        origin,
        input.enabled.unwrap_or(true),
        input.notes.unwrap_or_default().trim().chars().take(1000).collect(),
        Utc::now().to_rfc3339(),
    ))
}

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
                if name.as_ref().is_some_and(|v| v.len() > 100) {
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

fn reject_sensitive_input(value: &Value) -> Result<(), AppError> {
    match value {
        Value::Object(values) => {
            for (key, value) in values {
                if is_sensitive_key(key) {
                    return Err(AppError::InvalidInput(format!(
                        "input field '{key}' may contain a secret and is not supported"
                    )));
                }
                reject_sensitive_input(value)?;
            }
        }
        Value::Array(values) => {
            for value in values {
                reject_sensitive_input(value)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn reject_sensitive_template(value: &str) -> Result<(), AppError> {
    let mut remaining = value;
    while let Some(start) = remaining.find("{{input.") {
        let tail = &remaining[start + 8..];
        let Some(end) = tail.find("}}") else { break };
        let key = &tail[..end];
        if is_sensitive_key(key) {
            return Err(AppError::InvalidInput(format!(
                "input field '{key}' may contain a secret and is not supported"
            )));
        }
        remaining = &tail[end + 2..];
    }
    Ok(())
}

fn is_sensitive_key(key: &str) -> bool {
    let normalized = key
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>();
    if [
        "password",
        "passwd",
        "token",
        "secret",
        "credential",
        "authorization",
        "cookie",
        "apikey",
        "privatekey",
        "bearer",
        "passcode",
    ]
    .into_iter()
    .any(|word| normalized.starts_with(word) || normalized.ends_with(word))
    {
        return true;
    }
    let mut words = Vec::new();
    let mut word = String::new();
    let mut previous_was_lowercase_or_digit = false;
    for character in key.chars() {
        if !character.is_ascii_alphanumeric() {
            if !word.is_empty() {
                words.push(std::mem::take(&mut word));
            }
            previous_was_lowercase_or_digit = false;
            continue;
        }
        if character.is_ascii_uppercase() && previous_was_lowercase_or_digit && !word.is_empty() {
            words.push(std::mem::take(&mut word));
        }
        word.push(character.to_ascii_lowercase());
        previous_was_lowercase_or_digit =
            character.is_ascii_lowercase() || character.is_ascii_digit();
    }
    if !word.is_empty() {
        words.push(word);
    }

    words.iter().any(|word| {
        matches!(
            word.as_str(),
            "key"
                | "pwd"
                | "pass"
                | "auth"
                | "password"
                | "passwd"
                | "token"
                | "secret"
                | "credential"
                | "credentials"
                | "authorization"
                | "cookie"
                | "apikey"
                | "privatekey"
                | "bearer"
                | "passcode"
        )
    })
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

pub fn substitute(value: &str, input: &Value) -> Result<String, AppError> {
    let mut out = value.to_string();
    while let Some(start) = out.find("{{input.") {
        let tail = &out[start + 8..];
        let end = tail
            .find("}}")
            .ok_or_else(|| AppError::InvalidInput("invalid input variable".into()))?;
        let key = &tail[..end];
        let replacement = input
            .get(key)
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidInput(format!("missing string input: {key}")))?;
        out.replace_range(start..start + 8 + end + 2, replacement);
    }
    if out.contains("{{") || out.contains("}}") {
        return Err(AppError::InvalidInput("unsupported input variable".into()));
    }
    Ok(out)
}
fn required(value: String, name: &str, max: usize) -> Result<String, AppError> {
    let value = value.trim().to_string();
    if value.is_empty() || value.len() > max {
        Err(AppError::InvalidInput(format!("{name} must contain 1 to {max} characters")))
    } else {
        Ok(value)
    }
}
#[cfg(test)]
mod tests {
    use serde_json::json;
    use url::Url;

    use super::{goto_target, reject_sensitive_input, reject_sensitive_template, substitute};

    #[test]
    fn input_substitution_rejects_unsupported_or_missing_variables() {
        let input = json!({ "username": "owner", "password": "secret" });
        assert_eq!(
            substitute("{{input.username}}:{{input.password}}", &input).unwrap(),
            "owner:secret"
        );
        assert!(substitute("{{account.username}}", &input).is_err());
        assert!(substitute("{{input.missing}}", &input).is_err());
        assert!(substitute("{{input.username}", &input).is_err());
    }

    #[test]
    fn substituted_goto_target_must_remain_on_the_system_origin() {
        let base = Url::parse("https://fixture.local").expect("base URL");
        assert!(goto_target(&base, "/relative").is_ok());
        assert!(goto_target(&base, "https://other.local/from-input").is_err());
    }

    #[test]
    fn sensitive_input_is_rejected_at_nested_keys_and_template_definitions() {
        assert!(reject_sensitive_input(&json!({"profile": {"access_token": "secret"}})).is_err());
        assert!(reject_sensitive_template("{{input.clientSecret}}").is_err());
        assert!(reject_sensitive_input(&json!({"passwordValue": "secret"})).is_err());
        assert!(reject_sensitive_input(&json!({"token_value": "secret"})).is_err());
        assert!(reject_sensitive_template("{{input.apiKeyValue}}").is_err());
        for key in ["pwd", "passCode", "authHeader", "bearerToken", "user_pwd"] {
            let mut input = json!({});
            input.as_object_mut().expect("object").insert(key.into(), json!("secret"));
            assert!(reject_sensitive_input(&input).is_err(), "{key}");
        }
        for key in
            ["APIKey", "apikey", "privatekey", "passwordvalue", "JWTToken", "HTTPAuthorization"]
        {
            let mut input = json!({});
            input.as_object_mut().expect("object").insert(key.into(), json!("secret"));
            assert!(reject_sensitive_input(&input).is_err(), "{key}");
        }
        for key in [
            "userPasswordValue",
            "storedTokenValue",
            "clientSecretValue",
            "serviceCredentialValue",
            "sessionCookieValue",
        ] {
            let mut input = json!({});
            input.as_object_mut().expect("object").insert(key.into(), json!("secret"));
            assert!(reject_sensitive_input(&input).is_err(), "{key}");
        }
        for key in ["passengerCount", "passportNumber", "authorName", "authenticationStatus"] {
            let mut input = json!({});
            input.as_object_mut().expect("object").insert(key.into(), json!("allowed"));
            assert!(reject_sensitive_input(&input).is_ok(), "{key}");
        }
        assert!(reject_sensitive_input(&json!({"monkey": "allowed"})).is_ok());
        assert!(
            reject_sensitive_input(&json!({
                "passengerCount": 2,
                "passportNumber": "P123",
                "authorName": "Ada",
                "authenticationStatus": "verified",
            }))
            .is_ok()
        );
        assert!(reject_sensitive_input(&json!({"username": "owner", "value": "42"})).is_ok());
    }

    #[test]
    fn validate_steps_accepts_guard_press_and_pause() {
        use super::validate_flow;
        use crate::features::automation::types::{FlowStep, System};

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
