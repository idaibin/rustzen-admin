use serde_json::json;

use super::types::FlowStep;

#[test]
fn element_layout_step_round_trips_its_public_json_contract() {
    let value = json!({
        "action": "assertElementLayout",
        "selector": ".ant-table-thead th",
        "elementCount": 9,
        "visibleCount": 3,
        "maxHeight": 64,
        "withinViewportRight": true,
        "withinViewport": true,
    });
    let step: FlowStep = serde_json::from_value(value.clone()).expect("deserialize layout step");
    assert_eq!(step.action(), "assertElementLayout");
    assert_eq!(serde_json::to_value(step).expect("serialize layout step"), value);
}

#[test]
fn element_layout_step_serializes_default_and_missing_conditions_canonically() {
    let input = json!({
        "action": "assertElementLayout",
        "selector": ".recharts-line-dot",
        "elementCount": 2,
        "visibleCount": 2,
        "withinViewport": true,
    });
    let step: FlowStep = serde_json::from_value(input).expect("deserialize layout step");
    let expected = json!({
        "action": "assertElementLayout",
        "selector": ".recharts-line-dot",
        "elementCount": 2,
        "visibleCount": 2,
        "maxHeight": null,
        "withinViewportRight": false,
        "withinViewport": true,
    });
    assert_eq!(serde_json::to_value(step).expect("serialize layout step"), expected);
}
