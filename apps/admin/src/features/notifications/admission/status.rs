use super::AdmissionService;
use crate::features::notifications::{
    accounting::Accounting,
    admission_types::{AdmissionError, AdmissionStatus, CapacityReason, Failpoint},
    retention,
};

const RETRY_AFTER_SECONDS: u64 = 30;

impl AdmissionService {
    pub(super) fn check_capacity(
        &self,
        current: Accounting,
        messages: i64,
        recipients: i64,
        receipts: i64,
        charged_bytes: i64,
    ) -> Result<(), AdmissionError> {
        let reason = if current.message_count + messages > self.policy.message_limit {
            Some(CapacityReason::Messages)
        } else if current.recipient_count + recipients > self.policy.recipient_limit {
            Some(CapacityReason::Recipients)
        } else if current.receipt_count + receipts > self.policy.receipt_limit {
            Some(CapacityReason::Receipts)
        } else if current.charged_bytes + charged_bytes > self.policy.charged_bytes_limit {
            Some(CapacityReason::ChargedBytes)
        } else {
            None
        };
        reason.map_or(Ok(()), |reason| Err(self.capacity(reason)))
    }

    pub(super) fn capacity(&self, reason: CapacityReason) -> AdmissionError {
        *self.status.lock().expect("admission status lock") =
            AdmissionStatus { accepting: false, reason: Some(reason) };
        tracing::warn!(
            code = "notification-capacity",
            ?reason,
            retry_after_seconds = RETRY_AFTER_SECONDS
        );
        AdmissionError::Capacity { reason, retry_after_seconds: RETRY_AFTER_SECONDS }
    }

    pub(super) fn accepting(&self) {
        *self.status.lock().expect("admission status lock") =
            AdmissionStatus { accepting: true, reason: None };
    }

    pub(super) fn publish_cleanup(&self, result: &retention::CleanupResult) {
        if let Some(realtime) = &self.realtime {
            for &(user_id, revision) in &result.invalidations {
                realtime.publish(user_id, revision);
            }
        }
    }

    pub(super) fn fail(&self, stage: Failpoint) -> Result<(), AdmissionError> {
        #[cfg(test)]
        if self.failpoint == Some(stage) {
            return Err(AdmissionError::Database(sqlx::Error::Protocol(format!(
                "injected admission failure at {stage:?}"
            ))));
        }
        let _ = stage;
        Ok(())
    }
}
