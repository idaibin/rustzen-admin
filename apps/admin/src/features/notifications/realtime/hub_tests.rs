use super::hub::{Hub, Limits, RegisterError, Signal};
use std::sync::{Arc, Barrier};

#[tokio::test]
async fn quotas_are_atomic_and_release_exactly_once() {
    let hub = Hub::new(Limits { total: 2, per_user: 1, queue: 4 });
    let first = hub.register(1, 2).unwrap();
    assert_eq!(hub.counts(1), (1, 1));
    assert!(matches!(hub.register(1, 2), Err(RegisterError::UserLimit)));
    let second = hub.register(2, 3).unwrap();
    assert!(matches!(hub.register(3, 4), Err(RegisterError::GlobalLimit)));
    drop(first);
    assert_eq!(hub.counts(1), (1, 0));
    drop(second);
    assert_eq!(hub.counts(2), (0, 0));
}

#[test]
fn concurrent_registration_never_exceeds_user_or_global_quota() {
    let hub = Hub::new(Limits { total: 7, per_user: 4, queue: 16 });
    let barrier = Arc::new(Barrier::new(21));
    let mut workers = Vec::new();
    for index in 0..20 {
        let hub = hub.clone();
        let barrier = barrier.clone();
        workers.push(std::thread::spawn(move || {
            barrier.wait();
            hub.register(if index < 10 { 1 } else { 2 }, 0)
        }));
    }
    barrier.wait();
    let subscriptions =
        workers.into_iter().filter_map(|worker| worker.join().unwrap().ok()).collect::<Vec<_>>();
    assert_eq!(subscriptions.len(), 7);
    assert!(hub.counts(1).1 <= 4);
    assert!(hub.counts(2).1 <= 4);
    drop(subscriptions);
    assert_eq!(hub.counts(1), (0, 0));
}

#[tokio::test]
async fn lag_replaces_the_queue_with_reconcile_and_releases_quota() {
    let hub = Hub::new(Limits { total: 2, per_user: 2, queue: 2 });
    let stream = hub.register(4, 1).unwrap();
    hub.publish(4, 2);
    assert_eq!(hub.counts(4), (0, 0));
    assert_eq!(stream.next().await, Some(Signal::Reconcile("lagged")));
    assert_eq!(stream.next().await, None);
    drop(stream);
    assert_eq!(hub.counts(4), (0, 0));
}

#[tokio::test]
async fn shutdown_ends_streams_and_refuses_new_registrations() {
    let hub = Hub::new(Limits { total: 2, per_user: 2, queue: 4 });
    let stream = hub.register(1, 0).unwrap();
    hub.shutdown();
    assert_eq!(hub.counts(1), (0, 0));
    assert_eq!(stream.next().await, None);
    assert!(matches!(hub.register(1, 0), Err(RegisterError::Draining)));
}
