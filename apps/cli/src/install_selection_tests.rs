use super::*;
#[test]
fn notify_digest_is_target_exact_and_rejects_unsupported_target() {
    let x = digest("monitor-notify", "x86_64-unknown-linux-musl").unwrap();
    let a = digest("monitor-notify", "aarch64-unknown-linux-gnu").unwrap();
    assert_eq!(x, "225408b0137c11840e14983a9cafd2ab2477423834865998a30abaccf67fe464");
    assert_eq!(a, "d024bf9211a4984f1a9c1170583b7476f5ab246720c319e4c5791c9e33c1e310");
    assert_ne!(x, a);
    assert!(digest("monitor-notify", "bad").is_err());
}

#[test]
fn analytics_digest_is_target_exact_and_rejects_unsupported_target() {
    let x = digest("analytics", "x86_64-unknown-linux-musl").unwrap();
    let a = digest("analytics", "aarch64-unknown-linux-gnu").unwrap();
    assert_eq!(x, "799d104faafc7c0a04ae45211461fec03d159208f08194d181fdd02e0e0eedbb");
    assert_eq!(a, "5beed2b0836aa80b90d4b7f059ab41fc8866c68a49069dbc32446ed30996345c");
    assert_ne!(x, a);
    assert!(digest("analytics", "bad").is_err());
    assert!(digest("reports", "x86_64-unknown-linux-musl").is_err());
}
