use super::*;

const BASE: &str = "RUSTZEN_ENV=production\nRUSTZEN_JWT_SECRET=jwt-secret-012345\nRUSTZEN_IPC_TOKEN=ipc-token-012345\nRUSTZEN_MONITOR_AGENT_TOKEN=agent-token-012345\nRUSTZEN_ADMIN_SQLITE_PATH=/var/lib/rustzen-admin/admin.db\nRUSTZEN_MONITOR_SQLITE_PATH=/var/lib/rustzen-monitor/monitor.db\nRUSTZEN_ADMIN_RUNTIME_ROOT=/var/lib/rustzen-admin\nRUSTZEN_MONITOR_RUNTIME_ROOT=/var/lib/rustzen-monitor\nRUSTZEN_BOOTSTRAP_OWNER_PASSWORD=owner-password-012345\n";
const NOTIFY: &str = "RUSTZEN_NOTIFICATION_EVENT_KEY=notification-event-key-0123456789\nRUSTZEN_NOTIFICATION_EVENT_KEY_ID=notify-v1\nRUSTZEN_NOTIFICATION_INGRESS_PORT=19803\nRUSTZEN_NOTIFICATION_INGRESS_URL=http://127.0.0.1:19803/internal/v1/notification-events\nRUSTZEN_NOTIFICATION_CHARGED_BYTES_LIMIT=10\nRUSTZEN_NOTIFICATION_FREE_SPACE_RESERVE_BYTES=10\nRUSTZEN_NOTIFICATION_MESSAGE_LIMIT=10\nRUSTZEN_NOTIFICATION_RECEIPT_LIMIT=10\nRUSTZEN_NOTIFICATION_RECIPIENT_LIMIT=10\nRUSTZEN_NOTIFICATION_WAL_PRESSURE_FRAMES=10\nRUSTZEN_NOTIFICATION_WAL_PRESSURE_OBSERVATIONS=3\nRUSTZEN_NOTIFICATION_PREVIOUS_EVENT_KEY=previous-notification-event-key-012345\nRUSTZEN_NOTIFICATION_PREVIOUS_EVENT_KEY_ID=notify-old\nRUSTZEN_NOTIFICATION_PREVIOUS_EVENT_KEY_EXPIRES_AT=1\n";

#[test]
fn monitor_rejects_notification_keys() {
    assert!(parse_source(format!("{BASE}{NOTIFY}").as_bytes(), false).is_err());
}
#[test]
fn notify_requires_and_shares_notification_key_pair() {
    let values = parse_source(format!("{BASE}{NOTIFY}").as_bytes(), true).expect("notify values");
    let mut admin = values.clone();
    admin.insert("RUSTZEN_RUNTIME_ROOT".into(), "/var/lib/rustzen-admin".into());
    let mut monitor = values.clone();
    monitor.insert("RUSTZEN_RUNTIME_ROOT".into(), "/var/lib/rustzen-monitor".into());
    let admin = String::from_utf8(
        [render(&admin, ADMIN_KEYS).unwrap(), render(&admin, ADMIN_NOTIFY_KEYS).unwrap()].concat(),
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
    assert!(parse_source(missing.as_bytes(), true).is_err());
}
