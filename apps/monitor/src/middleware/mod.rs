use axum::http::HeaderMap;

use crate::{common::error::AppError, protocol::AGENT_REPORT_AUTH_HEADER};

pub fn require_agent_token(headers: &HeaderMap, expected: &str) -> Result<(), AppError> {
    let supplied = headers
        .get(AGENT_REPORT_AUTH_HEADER)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(AppError::invalid_agent_token)?;
    if constant_time_eq(supplied.as_bytes(), expected.as_bytes()) {
        Ok(())
    } else {
        Err(AppError::invalid_agent_token())
    }
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter().zip(right).fold(0_u8, |diff, (left, right)| diff | (left ^ right)) == 0
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue};

    use super::require_agent_token;
    use crate::protocol::AGENT_REPORT_AUTH_HEADER;

    #[test]
    fn agent_token_is_required_and_compared_exactly() {
        let mut headers = HeaderMap::new();
        assert!(require_agent_token(&headers, "secret").is_err());
        headers.insert(AGENT_REPORT_AUTH_HEADER, HeaderValue::from_static("wrong"));
        assert!(require_agent_token(&headers, "secret").is_err());
        headers.insert(AGENT_REPORT_AUTH_HEADER, HeaderValue::from_static("secret"));
        assert!(require_agent_token(&headers, "secret").is_ok());
    }
}
