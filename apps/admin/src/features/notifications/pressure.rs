use sqlx::SqlitePool;
use std::{io, path::Path, sync::Mutex};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum PressureReason {
    Checkpoint,
}

pub(crate) trait SpaceProbe: Send + Sync {
    fn available(&self, path: &Path) -> io::Result<u64>;
}

pub(crate) struct FilesystemSpace;

impl SpaceProbe for FilesystemSpace {
    fn available(&self, path: &Path) -> io::Result<u64> {
        fs2::available_space(path)
    }
}

#[derive(Debug, Default)]
pub(crate) struct PressureTracker {
    consecutive: Mutex<u32>,
}

impl PressureTracker {
    pub async fn observe_checkpoint(
        &self,
        pool: &SqlitePool,
        wal_frames: u32,
        observations: u32,
    ) -> Result<Option<PressureReason>, sqlx::Error> {
        let (busy, log_frames, checkpointed) =
            sqlx::query_as::<_, (i64, i64, i64)>("PRAGMA wal_checkpoint(PASSIVE)")
                .fetch_one(pool)
                .await?;
        let pressured =
            busy != 0 || log_frames.saturating_sub(checkpointed) >= i64::from(wal_frames);
        let mut consecutive = self.consecutive.lock().expect("pressure tracker lock");
        *consecutive = if pressured { consecutive.saturating_add(1) } else { 0 };
        Ok((*consecutive >= observations).then_some(PressureReason::Checkpoint))
    }
}
