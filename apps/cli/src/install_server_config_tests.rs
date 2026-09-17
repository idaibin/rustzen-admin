use super::*;

const BASE: &str = "RUSTZEN_ENV=production\nRUSTZEN_JWT_SECRET=jwt-secret-012345\nRUSTZEN_IPC_TOKEN=ipc-token-012345\nRUSTZEN_MONITOR_AGENT_TOKEN=agent-token-012345\nRUSTZEN_ADMIN_SQLITE_PATH=/var/lib/rustzen-admin/admin.db\nRUSTZEN_MONITOR_SQLITE_PATH=/var/lib/rustzen-monitor/monitor.db\nRUSTZEN_ADMIN_RUNTIME_ROOT=/var/lib/rustzen-admin\nRUSTZEN_MONITOR_RUNTIME_ROOT=/var/lib/rustzen-monitor\nRUSTZEN_BOOTSTRAP_OWNER_PASSWORD=owner-password-012345\n";
const NOTIFY: &str = "RUSTZEN_NOTIFICATION_EVENT_KEY=notification-event-key-0123456789\nRUSTZEN_NOTIFICATION_EVENT_KEY_ID=notify-v1\nRUSTZEN_NOTIFICATION_INGRESS_PORT=19803\nRUSTZEN_NOTIFICATION_INGRESS_URL=http://127.0.0.1:19803/internal/v1/notification-events\nRUSTZEN_NOTIFICATION_CHARGED_BYTES_LIMIT=10\nRUSTZEN_NOTIFICATION_FREE_SPACE_RESERVE_BYTES=10\nRUSTZEN_NOTIFICATION_MESSAGE_LIMIT=10\nRUSTZEN_NOTIFICATION_RECEIPT_LIMIT=10\nRUSTZEN_NOTIFICATION_RECIPIENT_LIMIT=10\nRUSTZEN_NOTIFICATION_WAL_PRESSURE_FRAMES=10\nRUSTZEN_NOTIFICATION_WAL_PRESSURE_OBSERVATIONS=3\nRUSTZEN_NOTIFICATION_PREVIOUS_EVENT_KEY=previous-notification-event-key-012345\nRUSTZEN_NOTIFICATION_PREVIOUS_EVENT_KEY_ID=notify-old\nRUSTZEN_NOTIFICATION_PREVIOUS_EVENT_KEY_EXPIRES_AT=1\n";
const ANALYTICS_BASE: &str = "RUSTZEN_ENV=production\nRUSTZEN_JWT_SECRET=jwt-secret-012345\nRUSTZEN_IPC_TOKEN=ipc-token-012345\nRUSTZEN_ADMIN_SQLITE_PATH=/var/lib/rustzen-admin/admin.db\nRUSTZEN_INSIGHTS_SQLITE_PATH=/var/lib/rustzen-insights/insights.db\nRUSTZEN_ADMIN_RUNTIME_ROOT=/var/lib/rustzen-admin\nRUSTZEN_INSIGHTS_RUNTIME_ROOT=/var/lib/rustzen-insights\nRUSTZEN_BOOTSTRAP_OWNER_PASSWORD=owner-password-012345\n";

fn monitor_selection() -> crate::install_server_selection::ServerSelection {
    crate::install_server_selection::for_preset("monitor").expect("monitor selection")
}
fn monitor_notify_selection() -> crate::install_server_selection::ServerSelection {
    crate::install_server_selection::for_preset("monitor-notify").expect("monitor-notify selection")
}
fn analytics_selection() -> crate::install_server_selection::ServerSelection {
    crate::install_server_selection::for_preset("analytics").expect("analytics selection")
}

#[test]
fn monitor_rejects_notification_keys() {
    assert!(parse_source(format!("{BASE}{NOTIFY}").as_bytes(), &monitor_selection()).is_err());
}
#[test]
fn notify_requires_and_shares_notification_key_pair() {
    let values = parse_source(format!("{BASE}{NOTIFY}").as_bytes(), &monitor_notify_selection())
        .expect("notify values");
    let mut admin = values.clone();
    admin.insert("RUSTZEN_RUNTIME_ROOT".into(), "/var/lib/rustzen-admin".into());
    let mut monitor = values.clone();
    monitor.insert("RUSTZEN_RUNTIME_ROOT".into(), "/var/lib/rustzen-monitor".into());
    let admin = String::from_utf8(
        [render(&admin, ADMIN_MONITOR_KEYS).unwrap(), render(&admin, ADMIN_NOTIFY_KEYS).unwrap()]
            .concat(),
    )
    .unwrap();
    let monitor = String::from_utf8(
        [render(&monitor, MONITOR_KEYS).unwrap(), render(&monitor, MONITOR_NOTIFY_KEYS).unwrap()]
            .concat(),
    )
    .unwrap();
    for key in ["RUSTZEN_NOTIFICATION_EVENT_KEY", "RUSTZEN_NOTIFICATION_EVENT_KEY_ID"] {
        let value = values.get(key).unwrap();
        assert!(admin.contains(&format!("{key}={value}\n")));
        assert!(monitor.contains(&format!("{key}={value}\n")));
    }
    let missing = format!(
        "{BASE}{}",
        NOTIFY.replace("RUSTZEN_NOTIFICATION_EVENT_KEY=notification-event-key-0123456789\n", "")
    );
    assert!(parse_source(missing.as_bytes(), &monitor_notify_selection()).is_err());
}

#[test]
fn analytics_accepts_only_insights_pair_keys() {
    let values =
        parse_source(ANALYTICS_BASE.as_bytes(), &analytics_selection()).expect("analytics values");
    assert_eq!(
        values.get("RUSTZEN_INSIGHTS_SQLITE_PATH").unwrap(),
        "/var/lib/rustzen-insights/insights.db"
    );
    let mut admin = values.clone();
    admin.insert("RUSTZEN_RUNTIME_ROOT".into(), "/var/lib/rustzen-admin".into());
    let mut insights = values.clone();
    insights.insert("RUSTZEN_RUNTIME_ROOT".into(), "/var/lib/rustzen-insights".into());
    let admin = String::from_utf8(render(&admin, ADMIN_INSIGHTS_KEYS).unwrap()).unwrap();
    let insights = String::from_utf8(render(&insights, INSIGHTS_KEYS).unwrap()).unwrap();
    assert!(
        admin.contains("RUSTZEN_INSIGHTS_PORT=9802\n") || !admin.contains("RUSTZEN_INSIGHTS_PORT")
    );
    assert!(!admin.contains("RUSTZEN_MONITOR_PORT"));
    assert!(!insights.contains("RUSTZEN_MONITOR"));
    assert!(parse_source(BASE.as_bytes(), &analytics_selection()).is_err());
    assert!(
        parse_source(
            format!("{ANALYTICS_BASE}RUSTZEN_MONITOR_AGENT_TOKEN=x\n").as_bytes(),
            &analytics_selection()
        )
        .is_err()
    );
}
