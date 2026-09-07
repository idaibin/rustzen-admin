use super::{
    accounting::{Accounting, USER_STATE_CHARGE, message_charge, receipt_charge, recipient_charge},
    admission_repo,
    admission_types::*,
    pressure::{FilesystemSpace, PressureReason, PressureTracker, SpaceProbe},
    realtime::RealtimeHub,
    retention,
};
use chrono::NaiveDateTime;
use sqlx::SqlitePool;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

const MAX_CLEANUP_ROUNDS: usize = 8;

mod status;

pub(crate) struct AdmissionService {
    pool: SqlitePool,
    database_path: PathBuf,
    policy: AdmissionPolicy,
    pressure: PressureTracker,
    space: Arc<dyn SpaceProbe>,
    status: Mutex<AdmissionStatus>,
    realtime: Option<RealtimeHub>,
    #[cfg(test)]
    failpoint: Option<Failpoint>,
}

impl AdmissionService {
    pub async fn start(
        pool: SqlitePool,
        database_path: PathBuf,
        policy: AdmissionPolicy,
    ) -> Result<Self, AdmissionError> {
        Self::start_with_probe(pool, database_path, policy, Arc::new(FilesystemSpace), None).await
    }

    pub(crate) async fn start_with_realtime(
        pool: SqlitePool,
        database_path: PathBuf,
        policy: AdmissionPolicy,
        realtime: RealtimeHub,
    ) -> Result<Self, AdmissionError> {
        Self::start_with_probe(
            pool,
            database_path,
            policy,
            Arc::new(FilesystemSpace),
            Some(realtime),
        )
        .await
    }

    async fn start_with_probe(
        pool: SqlitePool,
        database_path: PathBuf,
        policy: AdmissionPolicy,
        space: Arc<dyn SpaceProbe>,
        realtime: Option<RealtimeHub>,
    ) -> Result<Self, AdmissionError> {
        if policy.message_limit <= 0
            || policy.recipient_limit <= 0
            || policy.receipt_limit <= 0
            || policy.charged_bytes_limit <= 0
            || policy.wal_pressure_frames == 0
            || policy.wal_pressure_observations == 0
        {
            return Err(AdmissionError::Invalid);
        }
        let mut connection = pool.acquire().await?;
        Accounting::validate(&mut connection).await.map_err(|_| AdmissionError::Accounting)?;
        drop(connection);
        Ok(Self {
            pool,
            database_path,
            policy,
            pressure: PressureTracker::default(),
            space,
            status: Mutex::new(AdmissionStatus { accepting: true, reason: None }),
            realtime,
            #[cfg(test)]
            failpoint: None,
        })
    }

    #[cfg(test)]
    pub(super) async fn start_for_test(
        pool: SqlitePool,
        database_path: PathBuf,
        policy: AdmissionPolicy,
        space: Arc<dyn SpaceProbe>,
    ) -> Result<Self, AdmissionError> {
        Self::start_with_probe(pool, database_path, policy, space, None).await
    }

    #[cfg(test)]
    pub(super) fn with_failpoint(mut self, failpoint: Failpoint) -> Self {
        self.failpoint = Some(failpoint);
        self
    }

    pub fn status(&self) -> AdmissionStatus {
        *self.status.lock().expect("admission status lock")
    }

    pub async fn admit(
        &self,
        event: &AdmissionEvent,
        accepted_at: NaiveDateTime,
    ) -> Result<AdmissionResult, AdmissionError> {
        self.admit_inner(event, accepted_at, false).await
    }

    pub(crate) async fn admit_current_monitor_audience(
        &self,
        event: &AdmissionEvent,
        accepted_at: NaiveDateTime,
    ) -> Result<AdmissionResult, AdmissionError> {
        self.admit_inner(event, accepted_at, true).await
    }

    async fn admit_inner(
        &self,
        event: &AdmissionEvent,
        accepted_at: NaiveDateTime,
        current_monitor_audience: bool,
    ) -> Result<AdmissionResult, AdmissionError> {
        admission_repo::validate_key(event)?;
        let mut precheck = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        if let Some((digest, result)) = admission_repo::duplicate(&mut precheck, event).await? {
            return if digest == event.payload_sha256 {
                precheck.commit().await?;
                Ok(AdmissionResult::Duplicate { result })
            } else {
                precheck.rollback().await?;
                Err(AdmissionError::Conflict)
            };
        }
        precheck.commit().await?;
        admission_repo::validate_new(event, accepted_at)?;

        for _ in 0..MAX_CLEANUP_ROUNDS {
            let mut cleanup = self.pool.begin_with("BEGIN IMMEDIATE").await?;
            if let Some((digest, result)) = admission_repo::duplicate(&mut cleanup, event).await? {
                return if digest == event.payload_sha256 {
                    cleanup.commit().await?;
                    Ok(AdmissionResult::Duplicate { result })
                } else {
                    cleanup.rollback().await?;
                    Err(AdmissionError::Conflict)
                };
            }
            let cleaned =
                retention::cleanup(&mut cleanup, accepted_at, self.policy.cleanup_time_limit)
                    .await?;
            cleanup.commit().await?;
            self.publish_cleanup(&cleaned);
            if !cleaned.made_progress() {
                break;
            }
        }

        let pressure = self
            .pressure
            .observe_checkpoint(
                &self.pool,
                self.policy.wal_pressure_frames,
                self.policy.wal_pressure_observations,
            )
            .await?;
        let mut transaction = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        if let Some((digest, result)) = admission_repo::duplicate(&mut transaction, event).await? {
            return if digest == event.payload_sha256 {
                transaction.commit().await?;
                Ok(AdmissionResult::Duplicate { result })
            } else {
                transaction.rollback().await?;
                Err(AdmissionError::Conflict)
            };
        }
        if pressure == Some(PressureReason::Checkpoint) {
            transaction.rollback().await?;
            return Err(self.capacity(CapacityReason::Checkpoint));
        }
        let recipients = if current_monitor_audience {
            admission_repo::eligible_current_monitor(&mut transaction).await?
        } else {
            admission_repo::eligible(&mut transaction, event).await?
        };
        if recipients.len() > 1_000 {
            transaction.rollback().await?;
            return Err(AdmissionError::AudienceTooLarge);
        }
        let notification_id = Uuid::new_v4().to_string();
        let receipt_bytes = receipt_charge(&event.producer, &event.event_id, &event.payload_sha256);
        let message_bytes = if recipients.is_empty() { 0 } else { message_charge(event) };
        let recipient_bytes = recipient_charge(&notification_id) * recipients.len() as i64;
        let state_bytes = admission_repo::missing_states(&mut transaction, &recipients).await?
            * USER_STATE_CHARGE;
        let projected = receipt_bytes + message_bytes + recipient_bytes + state_bytes;
        self.check_capacity(
            admission_repo::accounting(&mut transaction).await?,
            (!recipients.is_empty()) as i64,
            recipients.len() as i64,
            1,
            projected,
        )?;
        let required = self
            .policy
            .free_space_reserve_bytes
            .checked_add(u64::try_from(projected).map_err(|_| AdmissionError::Invalid)?)
            .ok_or_else(|| self.capacity(CapacityReason::FreeSpace))?;
        let available = self
            .space
            .available(&self.database_path)
            .map_err(|error| AdmissionError::Database(sqlx::Error::Io(error)))?;
        if available < required {
            transaction.rollback().await?;
            return Err(self.capacity(CapacityReason::FreeSpace));
        }
        let result = if recipients.is_empty() { "no-recipients" } else { "stored" };
        let retain_until = std::cmp::max(
            accepted_at + chrono::Duration::days(retention::RETENTION_DAYS),
            event.expires_at + chrono::Duration::seconds(60),
        );
        sqlx::query(
            "INSERT INTO notification_receipts
             (producer, event_id, payload_sha256, accepted_at, expires_at, retain_until, result, charged_bytes)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&event.producer).bind(&event.event_id).bind(&event.payload_sha256)
        .bind(accepted_at).bind(event.expires_at).bind(retain_until).bind(result)
        .bind(receipt_bytes).execute(&mut *transaction).await?;
        self.fail(Failpoint::AfterReceipt)?;
        if recipients.is_empty() {
            transaction.commit().await?;
            self.accepting();
            return Ok(AdmissionResult::NoRecipients);
        }
        admission_repo::insert_message(
            &mut transaction,
            &notification_id,
            event,
            accepted_at,
            message_bytes,
        )
        .await?;
        self.fail(Failpoint::AfterMessage)?;
        let mut invalidations = Vec::with_capacity(recipients.len());
        for (index, user_id) in recipients.iter().enumerate() {
            let revision = sqlx::query_scalar::<_, i64>(
                "INSERT INTO notification_user_state (user_id, revision, charged_bytes)
                 VALUES (?, 1, ?) ON CONFLICT(user_id) DO UPDATE SET revision = revision + 1
                 RETURNING revision",
            )
            .bind(user_id)
            .bind(USER_STATE_CHARGE)
            .fetch_one(&mut *transaction)
            .await?;
            invalidations.push((*user_id, revision));
            sqlx::query(
                "INSERT INTO notification_recipients
                 (notification_id, user_id, created_at, charged_bytes) VALUES (?, ?, ?, ?)",
            )
            .bind(&notification_id)
            .bind(user_id)
            .bind(accepted_at)
            .bind(recipient_charge(&notification_id))
            .execute(&mut *transaction)
            .await?;
            if index == 0 {
                self.fail(Failpoint::DuringRecipients)?;
            }
        }
        self.fail(Failpoint::AfterRecipients)?;
        transaction.commit().await?;
        if let Some(realtime) = &self.realtime {
            for (user_id, revision) in invalidations {
                realtime.publish(user_id, revision);
            }
        }
        self.accepting();
        Ok(AdmissionResult::Stored { notification_id, recipients: recipients.len() })
    }
}
