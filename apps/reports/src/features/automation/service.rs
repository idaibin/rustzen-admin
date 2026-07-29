use chrono::Utc;
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
    let contains_sensitive_name = [
        "password",
        "passwd",
        "token",
        "secret",
        "credential",
        "authorization",
        "cookie",
        "apikey",
        "privatekey",
    ]
    .into_iter()
    .any(|name| normalized.starts_with(name) || normalized.ends_with(name));
    if contains_sensitive_name {
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
                | "password"
                | "passwd"
                | "token"
                | "secret"
                | "credential"
                | "credentials"
                | "authorization"
                | "cookie"
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
        assert!(reject_sensitive_input(&json!({"monkey": "allowed"})).is_ok());
        assert!(reject_sensitive_input(&json!({"username": "owner", "value": "42"})).is_ok());
    }
}
