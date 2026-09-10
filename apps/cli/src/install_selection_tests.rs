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
