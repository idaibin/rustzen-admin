use url::Url;

use super::{goto_target, validate_element_layout, validate_flow};
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
        FlowStep::GuardExists { selector: "#banner".into(), on_missing: Some("skipNext".into()) },
        FlowStep::Click { selector: "#banner-close".into() },
        FlowStep::Fill { selector: "#kw".into(), value: "test".into() },
        FlowStep::AssertValue { selector: "#kw".into(), value: "test".into() },
        FlowStep::AssertAbsent { selector: "#missing".into() },
        FlowStep::AssertFocus { selector: "#schedule-create".into() },
        FlowStep::PressKey { key: "Enter".into() },
        FlowStep::Pause { duration_ms: 500 },
        FlowStep::Screenshot { name: Some("result".into()) },
        FlowStep::ScreenshotViewport { name: Some("viewport".into()) },
        FlowStep::SetViewport { width: 390, height: 844 },
        FlowStep::SetUiPreferences { theme: "light".into(), locale: "zh-CN".into() },
        FlowStep::AssertNoHorizontalOverflow,
        FlowStep::AssertElementLayout {
            selector: ".table th".into(),
            element_count: Some(9),
            visible_count: Some(3),
            max_height: Some(64),
            within_viewport_right: true,
            within_viewport: false,
        },
    ];
    assert!(validate_flow(&system, &steps).is_ok());
    assert_eq!(steps[6].action(), "assertFocus");
    assert_eq!(steps.last().map(FlowStep::action), Some("assertElementLayout"));
    for selector in ["[data-testid=schedule-panel]", "//button", "xpath=//button"] {
        assert!(
            validate_flow(&system, &[FlowStep::AssertFocus { selector: selector.into() }]).is_ok()
        );
    }

    let invalid_guard = vec![FlowStep::GuardExists {
        selector: "#test".into(),
        on_missing: Some("invalid_strategy".into()),
    }];
    assert!(validate_flow(&system, &invalid_guard).is_err());
    assert!(validate_flow(&system, &[FlowStep::AssertFocus { selector: "".into() }]).is_err());

    assert!(validate_flow(&system, &[FlowStep::SetViewport { width: 400, height: 800 }],).is_err());
}

#[test]
fn validate_pause_allows_its_inclusive_bound() {
    let system = System {
        id: "system".into(),
        name: "System".into(),
        base_url: "https://fixture.local".into(),
        enabled: true,
        notes: String::new(),
        created_at: String::new(),
        updated_at: String::new(),
    };

    assert!(validate_flow(&system, &[FlowStep::Pause { duration_ms: 0 }]).is_ok());
    assert!(validate_flow(&system, &[FlowStep::Pause { duration_ms: 30_000 }]).is_ok());
    assert!(validate_flow(&system, &[FlowStep::Pause { duration_ms: 30_001 }]).is_err());
}

#[test]
fn validate_element_layout_requires_css_and_a_bounded_condition() {
    assert!(validate_element_layout(".table", Some(9), Some(3), Some(64), true, false).is_ok());
    assert!(validate_element_layout(".table", None, None, None, false, false).is_err());
    assert!(validate_element_layout(".table", None, None, Some(0), false, false).is_err());
    assert!(validate_element_layout("//table", None, None, Some(64), false, false).is_err());
}
