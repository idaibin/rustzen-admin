use super::*;
#[test]
fn selected_activation_owner_sets_are_exact() {
    assert_eq!(owners_for_preset("monitor").unwrap(), vec!["admin", "monitor"]);
    assert_eq!(
        owners_for_preset("monitor-notify").unwrap(),
        vec!["admin", "admin-notifications", "monitor", "monitor-notifications"]
    );
    assert!(owners_for_preset("reports").is_err());
}
