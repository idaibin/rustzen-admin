use std::time::{Duration, Instant};

use chromiumoxide::page::Page;
use serde_json::Value;

use crate::common::error::AppError;

pub(super) fn assert_no_horizontal_overflow(metrics: Option<&Value>) -> Result<(), AppError> {
    let metrics = metrics
        .and_then(Value::as_object)
        .ok_or_else(|| AppError::internal("browser did not report layout dimensions"))?;
    let scroll_width = metrics
        .get("scrollWidth")
        .and_then(Value::as_u64)
        .filter(|value| *value > 0)
        .ok_or_else(|| AppError::internal("browser scroll width is invalid"))?;
    let inner_width = metrics
        .get("innerWidth")
        .and_then(Value::as_u64)
        .filter(|value| *value > 0)
        .ok_or_else(|| AppError::internal("browser inner width is invalid"))?;
    if scroll_width > inner_width {
        return Err(AppError::Conflict("page has horizontal overflow".into()));
    }
    Ok(())
}

pub(super) fn fill_script(value: &str) -> Result<String, serde_json::Error> {
    let encoded = serde_json::to_string(value)?;
    Ok(format!(
        "function() {{ const prototype = this instanceof HTMLInputElement ? HTMLInputElement.prototype : this instanceof HTMLTextAreaElement ? HTMLTextAreaElement.prototype : this instanceof HTMLSelectElement ? HTMLSelectElement.prototype : null; const setter = prototype && Object.getOwnPropertyDescriptor(prototype, 'value')?.set; if (!setter) throw new Error('element has no native value setter'); setter.call(this, {encoded}); this.dispatchEvent(new Event('input', {{ bubbles: true }})); this.dispatchEvent(new Event('change', {{ bubbles: true }})); }}"
    ))
}

pub(super) fn is_xpath(selector: &str) -> bool {
    selector.starts_with("//") || selector.starts_with("xpath=")
}

pub(super) async fn locate_element(
    page: &Page,
    selector: &str,
) -> Result<chromiumoxide::Element, AppError> {
    if is_xpath(selector) {
        let clean = selector.strip_prefix("xpath=").unwrap_or(selector);
        page.find_xpath(clean).await.map_err(AppError::internal)
    } else {
        page.find_element(selector).await.map_err(AppError::internal)
    }
}

pub(super) async fn wait_for(page: &Page, selector: &str) -> Result<(), AppError> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let found = if is_xpath(selector) {
            locate_element(page, selector).await.is_ok()
        } else {
            let script = selector_exists_script(selector)?;
            page.evaluate(script)
                .await
                .ok()
                .and_then(|result| result.value().and_then(Value::as_bool))
                .unwrap_or(false)
        };
        if found {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(AppError::Conflict("waitFor selector timed out".into()));
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

pub(super) fn selector_exists_script(selector: &str) -> Result<String, AppError> {
    let selector = serde_json::to_string(selector).map_err(AppError::internal)?;
    Ok(format!("Boolean(document.querySelector({selector}))"))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{assert_no_horizontal_overflow, fill_script, is_xpath, selector_exists_script};

    #[test]
    fn xpath_selector_detection_supports_slash_and_prefix() {
        assert!(is_xpath("//button[@id='su']"));
        assert!(is_xpath("//*[@id='kw']"));
        assert!(is_xpath("xpath=//input"));
        assert!(is_xpath("xpath=//*[@class='title']"));
        assert!(!is_xpath("#kw"));
        assert!(!is_xpath("button.submit"));
        assert!(!is_xpath("[data-testid='btn']"));
    }

    #[test]
    fn css_selector_is_json_encoded_for_live_document_queries() {
        let script = selector_exists_script("button[data-name='a\\\"b']").unwrap();
        assert_eq!(script, "Boolean(document.querySelector(\"button[data-name='a\\\\\\\"b']\"))");
    }

    #[test]
    fn horizontal_overflow_requires_unsigned_dimensions() {
        assert!(
            assert_no_horizontal_overflow(Some(&json!({"scrollWidth":390,"innerWidth":390})))
                .is_ok()
        );
        assert!(
            assert_no_horizontal_overflow(Some(&json!({"scrollWidth":391,"innerWidth":390})))
                .is_err()
        );
        assert!(
            assert_no_horizontal_overflow(Some(&json!({"scrollWidth":"390","innerWidth":390})))
                .is_err()
        );
        assert!(assert_no_horizontal_overflow(Some(&json!({"scrollWidth":390}))).is_err());
        assert!(
            assert_no_horizontal_overflow(Some(&json!({"scrollWidth":0,"innerWidth":0}))).is_err()
        );
        assert!(
            assert_no_horizontal_overflow(Some(&json!({"scrollWidth":0,"innerWidth":390})))
                .is_err()
        );
        assert!(
            assert_no_horizontal_overflow(Some(&json!({"scrollWidth":390,"innerWidth":0})))
                .is_err()
        );
        assert!(assert_no_horizontal_overflow(Some(&json!([]))).is_err());
    }

    #[test]
    fn fill_uses_the_native_value_setter_and_json_escapes_input() {
        let script = fill_script("quoted \"value\"\nnext").expect("fill script");
        assert!(script.contains("this instanceof HTMLInputElement"));
        assert!(script.contains("this instanceof HTMLTextAreaElement"));
        assert!(script.contains("this instanceof HTMLSelectElement"));
        assert!(script.contains("Object.getOwnPropertyDescriptor(prototype, 'value')?.set"));
        assert!(script.contains("setter.call(this, \"quoted \\\"value\\\"\\nnext\")"));
        assert!(script.contains("new Event('input', { bubbles: true })"));
        assert!(script.contains("new Event('change', { bubbles: true })"));
    }
}
