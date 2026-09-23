use super::{Clock, RealtimeHub, Timing, hub::Limits};
use crate::features::notifications::admission_types::{AdmissionEvent, AdmissionPolicy};
use axum::{
    Extension, Router,
    http::{Request, header::AUTHORIZATION},
    middleware,
};
use rustzen_auth::auth::{AuthClaims, JwtCodec, auth_middleware};
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicI64, Ordering},
    },
    time::Duration,
};
use tower::ServiceExt;

pub(super) fn event(
    now: chrono::NaiveDateTime,
    event_id: &str,
    candidate_user_ids: Vec<i64>,
) -> AdmissionEvent {
    AdmissionEvent {
        producer: "monitor".into(),
        event_id: event_id.into(),
        payload_sha256: "a".repeat(64),
        expires_at: now + chrono::Duration::hours(1),
        topic: "monitor.incident.opened".into(),
        subject_kind: "incident".into(),
        subject_id: event_id.into(),
        subject_revision: 1,
        occurred_at: now,
        title: "incident opened".into(),
        summary: "summary".into(),
        required_capability: "monitor:incident:view".into(),
        candidate_user_ids,
    }
}

pub(super) fn policy() -> AdmissionPolicy {
    AdmissionPolicy {
        free_space_reserve_bytes: 0,
        wal_pressure_frames: u32::MAX,
        ..AdmissionPolicy::default()
    }
}

pub(super) struct Database {
    pub pool: SqlitePool,
    pub path: PathBuf,
}

impl Database {
    pub async fn new() -> Self {
        let path = std::env::temp_dir().join(format!("rustzen-sse-{}.db", uuid::Uuid::new_v4()));
        let options = SqliteConnectOptions::new()
            .filename(&path)
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_secs(2));
        let pool = SqlitePoolOptions::new().max_connections(4).connect_with(options).await.unwrap();
        crate::infra::db::run_migrations(&pool).await.unwrap();
        sqlx::query("UPDATE users SET status=1 WHERE id=1").execute(&pool).await.unwrap();
        Self { pool, path }
    }

    pub async fn add_user(&self, id: i64, name: &str, role: bool) {
        sqlx::query(
            "INSERT INTO users(id,username,email,password_hash,real_name,status,is_system)
             VALUES(?,?,?,'!test!',?,1,0)",
        )
        .bind(id)
        .bind(name)
        .bind(format!("{name}@invalid"))
        .bind(name)
        .execute(&self.pool)
        .await
        .unwrap();
        if role {
            sqlx::query("INSERT INTO user_roles(user_id,role_id) VALUES(?,1)")
                .bind(id)
                .execute(&self.pool)
                .await
                .unwrap();
        }
    }

    pub async fn close(self) {
        self.pool.close().await;
        let _ = std::fs::remove_file(&self.path);
        let _ = std::fs::remove_file(self.path.with_extension("db-wal"));
        let _ = std::fs::remove_file(self.path.with_extension("db-shm"));
    }
}

pub(super) fn codec() -> JwtCodec {
    JwtCodec::with_issuer_audience("sse-test-secret", 3_600, "sse-test", "rustzen-entry")
}

pub(super) async fn token(
    pool: &SqlitePool,
    codec: &JwtCodec,
    user_id: i64,
    username: &str,
    now: i64,
) -> (AuthClaims, String) {
    let epoch = sqlx::query_scalar::<_, i64>("SELECT auth_epoch FROM users WHERE id=?")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap();
    let sid = uuid::Uuid::new_v4().to_string();
    let claims = codec.claims_at(user_id, username, &sid, epoch, now);
    crate::features::auth::session::SessionRepository::create(
        pool,
        user_id,
        &sid,
        epoch,
        now,
        claims.exp as i64,
    )
    .await
    .unwrap();
    let encoded = codec.encode_claims(&claims).unwrap();
    (claims, encoded)
}

pub(super) fn realtime(
    pool: SqlitePool,
    total: usize,
    per_user: usize,
    queue: usize,
    clock: Clock,
) -> RealtimeHub {
    RealtimeHub::with_config(
        pool,
        Limits { total, per_user, queue },
        Timing {
            heartbeat: Duration::from_millis(20),
            recheck: Duration::from_millis(20),
            query_timeout: Duration::from_millis(500),
            poll_stall: Duration::from_secs(5),
            max_age: Duration::from_secs(10),
        },
        clock,
    )
}

pub(super) fn realtime_with_deadlines(
    pool: SqlitePool,
    poll_stall: Duration,
    max_age: Duration,
    clock: Clock,
) -> RealtimeHub {
    RealtimeHub::with_config(
        pool,
        Limits { total: 10, per_user: 4, queue: 16 },
        Timing {
            heartbeat: Duration::from_millis(10),
            recheck: Duration::from_millis(20),
            query_timeout: Duration::from_millis(500),
            poll_stall,
            max_age,
        },
        clock,
    )
}

pub(super) fn fixed_clock(now: i64) -> (Arc<AtomicI64>, Clock) {
    let value = Arc::new(AtomicI64::new(now));
    let clock_value = value.clone();
    (value, Arc::new(move || clock_value.load(Ordering::SeqCst)))
}

pub(super) fn app(pool: SqlitePool, realtime: RealtimeHub, codec: JwtCodec) -> Router {
    let (router, _) = crate::features::notifications::notification_routes().into_parts();
    router
        .layer(Extension(realtime))
        .route_layer(middleware::from_fn_with_state(
            (codec, crate::infra::auth_runtime::ServerAuthContextLoader::new(pool.clone())),
            auth_middleware,
        ))
        .with_state(pool)
}

pub(super) async fn request(
    app: Router,
    token: Option<&str>,
    uri: &str,
) -> axum::response::Response {
    let mut request = Request::get(uri);
    if let Some(token) = token {
        request = request.header(AUTHORIZATION, format!("Bearer {token}"));
    }
    app.oneshot(request.body(axum::body::Body::empty()).unwrap()).await.unwrap()
}
