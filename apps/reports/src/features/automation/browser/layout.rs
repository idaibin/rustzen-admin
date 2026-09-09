use chromiumoxide::Page;
use serde::Deserialize;
use serde_json::Value;

use crate::common::error::AppError;

#[derive(Clone, Copy)]
pub(super) struct ElementLayoutExpectation {
    pub(super) element_count: Option<u32>,
    pub(super) visible_count: Option<u32>,
    pub(super) max_height: Option<u32>,
    pub(super) within_viewport_right: bool,
    pub(super) within_viewport: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ElementLayoutMetrics {
    element_count: u32,
    visible_count: u32,
    max_height: Option<f64>,
    max_right: Option<f64>,
    min_left: Option<f64>,
    min_top: Option<f64>,
    max_bottom: Option<f64>,
    inner_width: f64,
    inner_height: f64,
}

pub(super) async fn assert_page_element_layout(
    page: &Page,
    selector: &str,
    element_count: Option<u32>,
    visible_count: Option<u32>,
    max_height: Option<u32>,
    within_viewport_right: bool,
    within_viewport: bool,
) -> Result<(), AppError> {
    let metrics =
        page.evaluate(element_layout_script(selector)?).await.map_err(AppError::internal)?;
    assert_element_layout(
        metrics.value(),
        ElementLayoutExpectation {
            element_count,
            visible_count,
            max_height,
            within_viewport_right,
            within_viewport,
        },
    )
}

pub(super) fn element_layout_script(selector: &str) -> Result<String, AppError> {
    let selector = serde_json::to_string(selector)?;
    Ok(format!(
        r#"(() => {{
            const elements = Array.from(document.querySelectorAll({selector}));
            const visible = elements.flatMap((element) => {{
                const style = getComputedStyle(element);
                const rect = element.getBoundingClientRect();
                const shown = style.display !== 'none'
                    && style.visibility !== 'hidden'
                    && style.opacity !== '0'
                    && rect.width > 0
                    && rect.height > 0;
                return shown ? [{{ height: rect.height, left: rect.left, top: rect.top, right: rect.right, bottom: rect.bottom }}] : [];
            }});
            return {{
                elementCount: elements.length,
                visibleCount: visible.length,
                maxHeight: visible.length
                    ? Math.max(...visible.map((item) => item.height))
                    : null,
                maxRight: visible.length
                    ? Math.max(...visible.map((item) => item.right))
                    : null,
                minLeft: visible.length
                    ? Math.min(...visible.map((item) => item.left))
                    : null,
                minTop: visible.length
                    ? Math.min(...visible.map((item) => item.top))
                    : null,
                maxBottom: visible.length
                    ? Math.max(...visible.map((item) => item.bottom))
                    : null,
                innerWidth: window.innerWidth,
                innerHeight: window.innerHeight,
            }};
        }})()"#
    ))
}

pub(super) fn assert_element_layout(
    value: Option<&Value>,
    expected: ElementLayoutExpectation,
) -> Result<(), AppError> {
    let value = value
        .cloned()
        .ok_or_else(|| AppError::internal("browser did not report element layout"))?;
    let metrics: ElementLayoutMetrics = serde_json::from_value(value)
        .map_err(|_| AppError::internal("browser element layout is invalid"))?;
    if let Some(expected_count) = expected.element_count
        && expected_count != metrics.element_count
    {
        return Err(AppError::Conflict(format!(
            "assertElementLayout expected {} elements, found {}",
            expected_count, metrics.element_count
        )));
    }
    if let Some(expected_count) = expected.visible_count
        && expected_count != metrics.visible_count
    {
        return Err(AppError::Conflict(format!(
            "assertElementLayout expected {} visible elements, found {}",
            expected_count, metrics.visible_count
        )));
    }
    if let Some(maximum) = expected.max_height {
        let actual =
            metrics.max_height.filter(|value| value.is_finite() && *value > 0.0).ok_or_else(
                || AppError::Conflict("assertElementLayout found no visible height".into()),
            )?;
        if actual > f64::from(maximum) + 0.5 {
            return Err(AppError::Conflict(format!(
                "assertElementLayout height {actual:.1}px exceeds {maximum}px"
            )));
        }
    }
    if expected.within_viewport_right {
        let right = metrics.max_right.filter(|value| value.is_finite()).ok_or_else(|| {
            AppError::Conflict("assertElementLayout found no visible edge".into())
        })?;
        if !metrics.inner_width.is_finite()
            || metrics.inner_width <= 0.0
            || right > metrics.inner_width + 0.5
        {
            return Err(AppError::Conflict(format!(
                "assertElementLayout right edge {right:.1}px exceeds viewport {:.1}px",
                metrics.inner_width
            )));
        }
    }
    if expected.within_viewport {
        let edges = [metrics.min_left, metrics.min_top, metrics.max_right, metrics.max_bottom];
        if edges.iter().any(|edge| !edge.is_some_and(f64::is_finite))
            || !metrics.inner_width.is_finite()
            || !metrics.inner_height.is_finite()
            || metrics.inner_width <= 0.0
            || metrics.inner_height <= 0.0
            || metrics.min_left.is_some_and(|edge| edge < -0.5)
            || metrics.min_top.is_some_and(|edge| edge < -0.5)
            || metrics.max_right.is_some_and(|edge| edge > metrics.inner_width + 0.5)
            || metrics.max_bottom.is_some_and(|edge| edge > metrics.inner_height + 0.5)
        {
            return Err(AppError::Conflict(
                "assertElementLayout found an element outside the viewport".into(),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{ElementLayoutExpectation, assert_element_layout, element_layout_script};

    fn expectation() -> ElementLayoutExpectation {
        ElementLayoutExpectation {
            element_count: Some(9),
            visible_count: Some(3),
            max_height: Some(64),
            within_viewport_right: true,
            within_viewport: true,
        }
    }

    fn metrics(elements: u32, visible: u32, height: u32, right: u32) -> Value {
        json!({
            "elementCount": elements,
            "visibleCount": visible,
            "maxHeight": height,
            "maxRight": right,
            "minLeft": 0,
            "minTop": 0,
            "maxBottom": 844,
            "innerWidth": 390,
            "innerHeight": 844,
        })
    }

    #[test]
    fn element_layout_metrics_enforce_count_height_and_viewport_edge() {
        let valid = json!({
            "elementCount": 9,
            "visibleCount": 3,
            "maxHeight": 63.5,
            "maxRight": 389.5,
            "minLeft": 0.0,
            "minTop": 0.0,
            "maxBottom": 843.5,
            "innerWidth": 390.0,
            "innerHeight": 844.0,
        });
        assert!(assert_element_layout(Some(&valid), expectation()).is_ok());
        for invalid in [
            metrics(8, 3, 63, 389),
            metrics(9, 4, 63, 389),
            metrics(9, 3, 65, 389),
            metrics(9, 3, 63, 391),
        ] {
            assert!(assert_element_layout(Some(&invalid), expectation()).is_err());
        }
        assert!(assert_element_layout(None, expectation()).is_err());
        for invalid in [
            json!({"elementCount":9,"visibleCount":3,"maxHeight":63,"minLeft":-1,"minTop":0,"maxRight":389,"maxBottom":843,"innerWidth":390,"innerHeight":844}),
            json!({"elementCount":9,"visibleCount":3,"maxHeight":63,"minLeft":0,"minTop":-1,"maxRight":389,"maxBottom":843,"innerWidth":390,"innerHeight":844}),
            json!({"elementCount":9,"visibleCount":3,"maxHeight":63,"minLeft":0,"minTop":0,"maxRight":389,"maxBottom":845,"innerWidth":390,"innerHeight":844}),
        ] {
            assert!(assert_element_layout(Some(&invalid), expectation()).is_err());
        }
    }

    #[test]
    fn element_layout_measurement_script_escapes_the_css_selector() {
        let script = element_layout_script("[data-label='\"quoted\"']").expect("layout script");
        assert!(script.contains("document.querySelectorAll"));
        assert!(script.contains(r#"[data-label='\"quoted\"']"#));
    }
}
