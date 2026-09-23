use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::Notify;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Signal {
    Changed(i64),
    Reconcile(&'static str),
}

#[derive(Clone, Copy)]
pub(super) struct Limits {
    pub total: usize,
    pub per_user: usize,
    pub queue: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self { total: 1_000, per_user: 4, queue: 16 }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(super) enum RegisterError {
    Draining,
    UserLimit,
    GlobalLimit,
}

#[derive(Clone)]
pub(super) struct Hub(Arc<HubInner>);

struct HubInner {
    limits: Limits,
    state: Mutex<HubState>,
}

struct HubState {
    accepting: bool,
    next_id: u64,
    entries: HashMap<u64, Entry>,
    users: HashMap<i64, usize>,
}

struct Entry {
    user_id: i64,
    connection: Arc<Connection>,
}

struct QueueState {
    queue: VecDeque<Signal>,
    closed: bool,
    last_polled: Instant,
}

pub(super) struct Connection {
    capacity: usize,
    state: Mutex<QueueState>,
    changed: Notify,
    progress: Notify,
}

pub(super) struct Subscription {
    hub: Hub,
    id: u64,
    connection: Arc<Connection>,
}

impl Hub {
    pub(super) fn new(limits: Limits) -> Self {
        assert!(limits.total > 0 && limits.per_user > 0 && limits.queue >= 2);
        Self(Arc::new(HubInner {
            limits,
            state: Mutex::new(HubState {
                accepting: true,
                next_id: 1,
                entries: HashMap::new(),
                users: HashMap::new(),
            }),
        }))
    }

    pub(super) fn register(
        &self,
        user_id: i64,
        revision: i64,
    ) -> Result<Subscription, RegisterError> {
        let mut state = self.0.state.lock().expect("notification hub state");
        if !state.accepting {
            return Err(RegisterError::Draining);
        }
        if state.entries.len() >= self.0.limits.total {
            return Err(RegisterError::GlobalLimit);
        }
        if state.users.get(&user_id).copied().unwrap_or_default() >= self.0.limits.per_user {
            return Err(RegisterError::UserLimit);
        }
        let id = state.next_id;
        state.next_id = state.next_id.checked_add(1).expect("notification connection id");
        let connection = Arc::new(Connection::new(self.0.limits.queue, revision));
        state.entries.insert(id, Entry { user_id, connection: connection.clone() });
        *state.users.entry(user_id).or_default() += 1;
        Ok(Subscription { hub: self.clone(), id, connection })
    }

    pub(crate) fn publish(&self, user_id: i64, revision: i64) {
        let targets = {
            let state = self.0.state.lock().expect("notification hub state");
            state
                .entries
                .iter()
                .filter(|(_, entry)| entry.user_id == user_id)
                .map(|(id, entry)| (*id, entry.connection.clone()))
                .collect::<Vec<_>>()
        };
        for (id, connection) in targets {
            if connection.enqueue(revision) {
                self.release(id, false);
            }
        }
    }

    pub(super) fn close(&self, id: u64) {
        self.release(id, true);
    }

    pub(super) fn shutdown(&self) {
        let entries = {
            let mut state = self.0.state.lock().expect("notification hub state");
            state.accepting = false;
            state.users.clear();
            std::mem::take(&mut state.entries)
        };
        for entry in entries.into_values() {
            entry.connection.close(true);
        }
    }

    fn release(&self, id: u64, clear: bool) {
        let entry = {
            let mut state = self.0.state.lock().expect("notification hub state");
            let entry = state.entries.remove(&id);
            if let Some(entry) = &entry
                && let Some(count) = state.users.get_mut(&entry.user_id)
            {
                *count -= 1;
                if *count == 0 {
                    state.users.remove(&entry.user_id);
                }
            }
            entry
        };
        if let Some(entry) = entry {
            entry.connection.close(clear);
        }
    }

    #[cfg(test)]
    pub(super) fn counts(&self, user_id: i64) -> (usize, usize) {
        let state = self.0.state.lock().expect("notification hub state");
        (state.entries.len(), state.users.get(&user_id).copied().unwrap_or_default())
    }

    #[cfg(test)]
    pub(super) fn pending(&self, user_id: i64) -> usize {
        let state = self.0.state.lock().expect("notification hub state");
        state
            .entries
            .values()
            .filter(|entry| entry.user_id == user_id)
            .map(|entry| entry.connection.pending())
            .sum()
    }
}

impl Connection {
    fn new(capacity: usize, revision: i64) -> Self {
        Self {
            capacity,
            state: Mutex::new(QueueState {
                queue: VecDeque::from([Signal::Reconcile("connected"), Signal::Changed(revision)]),
                closed: false,
                last_polled: Instant::now(),
            }),
            changed: Notify::new(),
            progress: Notify::new(),
        }
    }

    fn enqueue(&self, revision: i64) -> bool {
        let mut state = self.state.lock().expect("notification connection queue");
        if state.closed {
            return false;
        }
        if state.queue.len() >= self.capacity {
            state.queue.clear();
            state.queue.push_back(Signal::Reconcile("lagged"));
            state.closed = true;
            drop(state);
            self.changed.notify_waiters();
            return true;
        }
        state.queue.push_back(Signal::Changed(revision));
        drop(state);
        self.changed.notify_one();
        false
    }

    fn close(&self, clear: bool) {
        let mut state = self.state.lock().expect("notification connection queue");
        if clear {
            state.queue.clear();
        }
        state.closed = true;
        drop(state);
        self.changed.notify_waiters();
    }

    async fn next(&self) -> Option<Signal> {
        loop {
            let notified = self.changed.notified();
            {
                let mut state = self.state.lock().expect("notification connection queue");
                if let Some(signal) = state.queue.pop_front() {
                    return Some(signal);
                }
                if state.closed {
                    return None;
                }
            }
            notified.await;
        }
    }

    pub(super) async fn closed(&self) {
        loop {
            let notified = self.changed.notified();
            if self.state.lock().expect("notification connection queue").closed {
                return;
            }
            notified.await;
        }
    }

    pub(super) fn mark_polled(&self) {
        self.state.lock().expect("notification connection queue").last_polled = Instant::now();
        self.progress.notify_one();
    }

    pub(super) fn poll_stalled(&self, limit: Duration) -> bool {
        self.state.lock().expect("notification connection queue").last_polled.elapsed() >= limit
    }

    pub(super) fn poll_remaining(&self, limit: Duration) -> Duration {
        limit.saturating_sub(
            self.state.lock().expect("notification connection queue").last_polled.elapsed(),
        )
    }

    pub(super) async fn progressed(&self) {
        self.progress.notified().await;
    }

    #[cfg(test)]
    fn pending(&self) -> usize {
        self.state.lock().expect("notification connection queue").queue.len()
    }
}

impl Subscription {
    pub(super) fn id(&self) -> u64 {
        self.id
    }

    pub(super) fn connection(&self) -> Arc<Connection> {
        self.connection.clone()
    }

    pub(super) async fn next(&self) -> Option<Signal> {
        self.connection.next().await
    }

    pub(super) fn mark_polled(&self) {
        self.connection.mark_polled();
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        self.hub.close(self.id);
    }
}
