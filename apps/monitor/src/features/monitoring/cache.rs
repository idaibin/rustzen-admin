use std::{future::Future, time::Duration};

use axum::body::Bytes;
use tokio::{
    sync::{Mutex, RwLock},
    time::Instant,
};

use crate::common::error::AppError;

const NODES_CACHE_TTL: Duration = Duration::from_millis(250);

#[derive(Clone)]
struct CachedNodes {
    loaded_at: Instant,
    body: Bytes,
}

pub(crate) struct NodesCache {
    ttl: Duration,
    current: RwLock<Option<CachedNodes>>,
    refresh: Mutex<()>,
}

impl Default for NodesCache {
    fn default() -> Self {
        Self { ttl: NODES_CACHE_TTL, current: RwLock::new(None), refresh: Mutex::new(()) }
    }
}

impl NodesCache {
    pub(crate) async fn get_or_load<F, Fut>(&self, load: F) -> Result<Bytes, AppError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Bytes, AppError>>,
    {
        if let Some(body) = self.fresh_body().await {
            return Ok(body);
        }

        let _refresh = self.refresh.lock().await;
        if let Some(body) = self.fresh_body().await {
            return Ok(body);
        }

        let body = load().await?;
        *self.current.write().await =
            Some(CachedNodes { loaded_at: Instant::now(), body: body.clone() });
        Ok(body)
    }

    pub(crate) async fn invalidate(&self) {
        let _refresh = self.refresh.lock().await;
        *self.current.write().await = None;
    }

    async fn fresh_body(&self) -> Option<Bytes> {
        let current = self.current.read().await;
        let cached = current.as_ref()?;
        (cached.loaded_at.elapsed() <= self.ttl).then(|| cached.body.clone())
    }

    #[cfg(test)]
    fn with_ttl(ttl: Duration) -> Self {
        Self { ttl, ..Self::default() }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use super::*;

    #[tokio::test]
    async fn concurrent_misses_share_one_refresh() {
        let cache = Arc::new(NodesCache::default());
        let loads = Arc::new(AtomicUsize::new(0));
        let mut tasks = Vec::new();
        for _ in 0..32 {
            let cache = cache.clone();
            let loads = loads.clone();
            tasks.push(tokio::spawn(async move {
                cache
                    .get_or_load(|| async move {
                        loads.fetch_add(1, Ordering::SeqCst);
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        Ok(Bytes::from_static(br#"{"nodeId":"one"}"#))
                    })
                    .await
                    .unwrap()
            }));
        }
        for task in tasks {
            assert_eq!(task.await.unwrap(), Bytes::from_static(br#"{"nodeId":"one"}"#));
        }
        assert_eq!(loads.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn invalidation_and_expiry_force_a_refresh() {
        let cache = NodesCache::with_ttl(Duration::from_millis(5));
        let loads = AtomicUsize::new(0);
        let load = || async {
            let value = loads.fetch_add(1, Ordering::SeqCst);
            Ok(Bytes::from(value.to_string()))
        };

        assert_eq!(cache.get_or_load(load).await.unwrap(), "0");
        assert_eq!(cache.get_or_load(load).await.unwrap(), "0");
        cache.invalidate().await;
        assert_eq!(cache.get_or_load(load).await.unwrap(), "1");
        tokio::time::sleep(Duration::from_millis(6)).await;
        assert_eq!(cache.get_or_load(load).await.unwrap(), "2");
    }

    #[tokio::test]
    async fn failed_refresh_is_not_cached() {
        let cache = NodesCache::default();
        assert!(
            cache.get_or_load(|| async { Err::<Bytes, _>(AppError::database()) }).await.is_err()
        );
        assert_eq!(
            cache.get_or_load(|| async { Ok(Bytes::from_static(b"recovered")) }).await.unwrap(),
            "recovered"
        );
    }

    #[tokio::test]
    async fn invalidation_cannot_be_overwritten_by_an_in_flight_refresh() {
        let cache = Arc::new(NodesCache::default());
        let started = Arc::new(tokio::sync::Notify::new());
        let release = Arc::new(tokio::sync::Notify::new());
        let refresh = {
            let cache = cache.clone();
            let started = started.clone();
            let release = release.clone();
            tokio::spawn(async move {
                cache
                    .get_or_load(|| async move {
                        started.notify_one();
                        release.notified().await;
                        Ok(Bytes::from_static(b"stale"))
                    })
                    .await
                    .unwrap()
            })
        };
        started.notified().await;
        let invalidation = {
            let cache = cache.clone();
            tokio::spawn(async move { cache.invalidate().await })
        };
        tokio::task::yield_now().await;
        release.notify_one();
        assert_eq!(refresh.await.unwrap(), "stale");
        invalidation.await.unwrap();
        assert_eq!(
            cache.get_or_load(|| async { Ok(Bytes::from_static(b"fresh")) }).await.unwrap(),
            "fresh"
        );
    }
}
