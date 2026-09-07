mod authority;
mod handler;
mod hub;

pub(crate) use handler::stream;

use authority::Snapshot;
use axum::response::sse::Event;
use hub::{Hub, Limits, RegisterError, Signal, Subscription};
use rustzen_auth::auth::AuthClaims;
use sqlx::SqlitePool;
use std::{
    convert::Infallible,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

type Clock = Arc<dyn Fn() -> i64 + Send + Sync>;

struct Timing {
    heartbeat: Duration,
    recheck: Duration,
    query_timeout: Duration,
    poll_stall: Duration,
    max_age: Duration,
}

#[derive(Clone)]
pub(crate) struct RealtimeHub {
    hub: Hub,
    pool: SqlitePool,
    clock: Clock,
    heartbeat: Duration,
    recheck_period: Duration,
    query_timeout: Duration,
    poll_stall: Duration,
    max_age: Duration,
    supervisors: Arc<AtomicUsize>,
}

pub(crate) struct RealtimeStream {
    subscription: Subscription,
    heartbeat: tokio::time::Interval,
    expires_at: i64,
    clock: Clock,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OpenError {
    BadRequest,
    Unauthorized,
    Forbidden,
    Draining,
    UserLimit,
    GlobalLimit,
    Unavailable,
}

impl RealtimeHub {
    pub(crate) fn new(pool: SqlitePool) -> Self {
        Self::with_config(
            pool,
            Limits::default(),
            Timing {
                heartbeat: Duration::from_secs(15),
                recheck: Duration::from_secs(4),
                query_timeout: Duration::from_secs(1),
                poll_stall: Duration::from_secs(45),
                max_age: Duration::from_secs(5 * 60),
            },
            Arc::new(|| chrono::Utc::now().timestamp()),
        )
    }

    fn with_config(pool: SqlitePool, limits: Limits, timing: Timing, clock: Clock) -> Self {
        assert!(timing.poll_stall > timing.heartbeat && timing.max_age > timing.poll_stall);
        Self {
            hub: Hub::new(limits),
            pool,
            clock,
            heartbeat: timing.heartbeat,
            recheck_period: timing.recheck,
            query_timeout: timing.query_timeout,
            poll_stall: timing.poll_stall,
            max_age: timing.max_age,
            supervisors: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub(crate) async fn subscribe(&self, claims: AuthClaims) -> Result<RealtimeStream, OpenError> {
        let now = (self.clock)();
        let snapshot =
            tokio::time::timeout(self.query_timeout, authority::load(&self.pool, &claims, now))
                .await
                .map_err(|_| OpenError::Unavailable)?
                .map_err(map_authority)?;
        let subscription =
            self.hub.register(claims.user_id, snapshot.revision).map_err(map_register)?;
        self.spawn_supervisor(
            subscription.id(),
            subscription.connection(),
            claims.clone(),
            snapshot,
        );
        let heartbeat =
            tokio::time::interval_at(tokio::time::Instant::now() + self.heartbeat, self.heartbeat);
        Ok(RealtimeStream {
            subscription,
            heartbeat,
            expires_at: claims.exp as i64,
            clock: self.clock.clone(),
        })
    }

    fn spawn_supervisor(
        &self,
        id: u64,
        connection: Arc<hub::Connection>,
        claims: AuthClaims,
        expected: Snapshot,
    ) {
        let this = self.clone();
        this.supervisors.fetch_add(1, Ordering::SeqCst);
        tokio::spawn(async move {
            let _task = SupervisorGuard(this.supervisors.clone());
            let maximum_age = tokio::time::sleep(this.max_age);
            let expires_in = claims.exp.saturating_sub((this.clock)() as usize) as u64;
            let expiry = tokio::time::sleep(Duration::from_secs(expires_in));
            tokio::pin!(maximum_age, expiry);
            let mut next_recheck = tokio::time::Instant::now() + this.recheck_period;
            loop {
                let now = (this.clock)();
                if now >= claims.exp as i64 {
                    this.hub.close(id);
                    return;
                }
                let recheck = tokio::time::sleep_until(next_recheck);
                let poll_stall = tokio::time::sleep(connection.poll_remaining(this.poll_stall));
                tokio::pin!(recheck, poll_stall);
                tokio::select! {
                    biased;
                    () = connection.closed() => return,
                    () = &mut expiry => {
                        this.hub.close(id);
                        return;
                    }
                    () = &mut maximum_age => {
                        this.hub.close(id);
                        return;
                    }
                    () = &mut recheck => {}
                    () = &mut poll_stall => {
                        if connection.poll_stalled(this.poll_stall) {
                            this.hub.close(id);
                            return;
                        }
                        continue;
                    }
                    () = connection.progressed() => continue,
                }
                let now = (this.clock)();
                if now >= claims.exp as i64 {
                    this.hub.close(id);
                    return;
                }
                let current = tokio::time::timeout(
                    this.query_timeout,
                    authority::load(&this.pool, &claims, now),
                )
                .await;
                match current {
                    Ok(Ok(current)) if current.same_authority(&expected) => {
                        next_recheck = tokio::time::Instant::now() + this.recheck_period;
                    }
                    _ => {
                        this.hub.close(id);
                        return;
                    }
                }
            }
        });
    }

    pub(crate) fn publish(&self, user_id: i64, revision: i64) {
        self.hub.publish(user_id, revision);
    }

    pub(crate) fn shutdown(&self) {
        self.hub.shutdown();
    }

    #[cfg(test)]
    fn counts(&self, user_id: i64) -> (usize, usize) {
        self.hub.counts(user_id)
    }

    #[cfg(test)]
    fn pending(&self, user_id: i64) -> usize {
        self.hub.pending(user_id)
    }

    #[cfg(test)]
    fn supervisor_count(&self) -> usize {
        self.supervisors.load(Ordering::SeqCst)
    }
}

struct SupervisorGuard(Arc<AtomicUsize>);

impl Drop for SupervisorGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

impl RealtimeStream {
    pub(crate) async fn next_event(&mut self) -> Option<Result<Event, Infallible>> {
        self.subscription.mark_polled();
        let signal = tokio::select! {
            biased;
            signal = self.subscription.next() => signal,
            _ = self.heartbeat.tick() => Some(Signal::Reconcile("heartbeat")),
        }?;
        if (self.clock)() >= self.expires_at {
            return None;
        }
        let event = match signal {
            Signal::Changed(revision) => Event::default()
                .event("inbox.changed")
                .id(revision.to_string())
                .data(format!(r#"{{"schemaVersion":1,"revision":{revision}}}"#)),
            Signal::Reconcile("heartbeat") => Event::default().comment("heartbeat"),
            Signal::Reconcile(reason) => Event::default()
                .event("reconcile.required")
                .data(format!(r#"{{"schemaVersion":1,"reason":"{reason}"}}"#)),
        };
        Some(Ok(event))
    }
}

fn map_authority(error: authority::Error) -> OpenError {
    match error {
        authority::Error::Unauthorized => OpenError::Unauthorized,
        authority::Error::Forbidden => OpenError::Forbidden,
        authority::Error::Unavailable => OpenError::Unavailable,
    }
}

fn map_register(error: RegisterError) -> OpenError {
    match error {
        RegisterError::Draining => OpenError::Draining,
        RegisterError::UserLimit => OpenError::UserLimit,
        RegisterError::GlobalLimit => OpenError::GlobalLimit,
    }
}

#[cfg(test)]
mod hub_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests_authority;
#[cfg(test)]
mod tests_http;
#[cfg(test)]
mod tests_publish;
#[cfg(test)]
mod tests_retention;
#[cfg(test)]
mod tests_stall;
