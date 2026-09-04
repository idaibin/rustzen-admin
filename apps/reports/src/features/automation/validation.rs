use serde_json::Value;

use crate::common::error::AppError;

pub(crate) fn reject_sensitive_input(value: &Value) -> Result<(), AppError> {
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

pub(crate) fn reject_sensitive_template(value: &str) -> Result<(), AppError> {
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
            .and_then(|value| value.as_str())
            .ok_or_else(|| AppError::InvalidInput(format!("missing string input: {key}")))?;
        out.replace_range(start..start + 8 + end + 2, replacement);
    }
    if out.contains("{{") || out.contains("}}") {
        return Err(AppError::InvalidInput("unsupported input variable".into()));
    }
    Ok(out)
}

pub(crate) fn required(value: String, name: &str, max: usize) -> Result<String, AppError> {
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

    use super::{reject_sensitive_input, reject_sensitive_template, substitute};

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
}
