use std::collections::HashSet;

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Reports may be at most five minutes away from the Controller receive time.
/// The bound is deliberately small for a 30-second reporting interval and
/// prevents a bad Agent clock from distorting retention and daily statistics.
pub const MAX_REPORT_CLOCK_SKEW_SECONDS: i64 = 5 * 60;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentReport {
    pub node_id: String,
    pub boot_id: Uuid,
    pub sequence: u64,
    pub hostname: String,
    pub agent_version: String,
    pub collected_at: DateTime<Utc>,
    pub cpu_percent: f64,
    pub memory: ByteUsage,
    pub disks: Vec<DiskUsage>,
}

impl AgentReport {
    pub fn validate(&self) -> Result<(), ReportValidationError> {
        if self.node_id.is_empty()
            || self.node_id.len() > 128
            || !self
                .node_id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        {
            return Err(ReportValidationError::InvalidNodeId);
        }
        if self.hostname.trim().is_empty() || self.hostname.chars().count() > 253 {
            return Err(ReportValidationError::InvalidHostname);
        }
        if self.agent_version.trim().is_empty() || self.agent_version.chars().count() > 64 {
            return Err(ReportValidationError::InvalidAgentVersion);
        }
        if self.sequence == 0 {
            return Err(ReportValidationError::InvalidSequence);
        }
        if self.sequence > i64::MAX as u64 {
            return Err(ReportValidationError::SequenceTooLarge);
        }
        if !self.cpu_percent.is_finite() || !(0.0..=100.0).contains(&self.cpu_percent) {
            return Err(ReportValidationError::InvalidCpuPercent);
        }
        self.memory.validate().map_err(|_| ReportValidationError::InvalidMemory)?;
        let mut mounts = HashSet::with_capacity(self.disks.len());
        for disk in &self.disks {
            let mount = disk.mount_point.trim();
            if mount.is_empty() || mount.chars().count() > 256 {
                return Err(ReportValidationError::InvalidMountPoint);
            }
            if !mounts.insert(mount) {
                return Err(ReportValidationError::DuplicateMountPoint);
            }
            disk.usage.validate().map_err(|_| ReportValidationError::InvalidDiskUsage)?;
        }
        Ok(())
    }

    pub fn validate_at(&self, received_at: DateTime<Utc>) -> Result<(), ReportValidationError> {
        self.validate()?;
        let skew = self.collected_at.signed_duration_since(received_at).abs();
        if skew > ChronoDuration::seconds(MAX_REPORT_CLOCK_SKEW_SECONDS) {
            return Err(ReportValidationError::TimestampOutsideAllowedSkew);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ByteUsage {
    pub used_bytes: u64,
    pub total_bytes: u64,
}

impl ByteUsage {
    fn validate(self) -> Result<(), ()> {
        if self.total_bytes == 0
            || self.used_bytes > self.total_bytes
            || self.used_bytes > i64::MAX as u64
            || self.total_bytes > i64::MAX as u64
        {
            return Err(());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskUsage {
    pub mount_point: String,
    #[serde(flatten)]
    pub usage: ByteUsage,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AgentReportStatus {
    Accepted,
    Duplicate,
    Stale,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ReportValidationError {
    InvalidNodeId,
    InvalidHostname,
    InvalidAgentVersion,
    InvalidSequence,
    SequenceTooLarge,
    InvalidCpuPercent,
    InvalidMemory,
    InvalidMountPoint,
    DuplicateMountPoint,
    InvalidDiskUsage,
    TimestampOutsideAllowedSkew,
}

impl std::fmt::Display for ReportValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidNodeId => "nodeId is invalid",
            Self::InvalidHostname => "hostname is invalid",
            Self::InvalidAgentVersion => "agentVersion is invalid",
            Self::InvalidSequence => "sequence must be greater than zero",
            Self::SequenceTooLarge => "sequence is outside the supported range",
            Self::InvalidCpuPercent => "cpuPercent must be finite and between 0 and 100",
            Self::InvalidMemory => "memory total must be positive and used must not exceed total",
            Self::InvalidMountPoint => "mountPoint is invalid",
            Self::DuplicateMountPoint => "mountPoint values must be unique",
            Self::InvalidDiskUsage => "disk total must be positive and used must not exceed total",
            Self::TimestampOutsideAllowedSkew => "collectedAt is outside the allowed clock skew",
        })
    }
}

impl std::error::Error for ReportValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn report() -> AgentReport {
        AgentReport {
            node_id: "edge-a".into(),
            boot_id: Uuid::new_v4(),
            sequence: 1,
            hostname: "edge-a.example".into(),
            agent_version: "0.5.0".into(),
            collected_at: Utc::now(),
            cpu_percent: 21.5,
            memory: ByteUsage { used_bytes: 50, total_bytes: 100 },
            disks: vec![
                DiskUsage {
                    mount_point: "/".into(),
                    usage: ByteUsage { used_bytes: 60, total_bytes: 100 },
                },
                DiskUsage {
                    mount_point: "/data".into(),
                    usage: ByteUsage { used_bytes: 70, total_bytes: 100 },
                },
            ],
        }
    }

    #[test]
    fn valid_report_round_trips_with_independent_disk_mounts() {
        let report = report();
        report.validate().expect("valid report");
        let value = serde_json::to_value(&report).expect("serialize report");
        assert_eq!(value["disks"][0]["mountPoint"], "/");
        assert_eq!(value["disks"][1]["mountPoint"], "/data");
        assert_eq!(serde_json::from_value::<AgentReport>(value).unwrap(), report);
    }

    #[test]
    fn rejects_invalid_identity_sequence_and_cpu() {
        for mutate in [
            |report: &mut AgentReport| report.node_id = "bad node".into(),
            |report: &mut AgentReport| report.sequence = 0,
            |report: &mut AgentReport| report.cpu_percent = f64::NAN,
            |report: &mut AgentReport| report.cpu_percent = 101.0,
        ] as [fn(&mut AgentReport); 4]
        {
            let mut input = report();
            mutate(&mut input);
            assert!(input.validate().is_err());
        }
        let mut too_large = report();
        too_large.sequence = i64::MAX as u64 + 1;
        assert_eq!(too_large.validate(), Err(ReportValidationError::SequenceTooLarge));
    }

    #[test]
    fn rejects_impossible_memory_and_disk_values() {
        let mut invalid_memory = report();
        invalid_memory.memory.used_bytes = 101;
        assert_eq!(invalid_memory.validate(), Err(ReportValidationError::InvalidMemory));

        let mut invalid_disk = report();
        invalid_disk.disks[0].usage.total_bytes = 0;
        assert_eq!(invalid_disk.validate(), Err(ReportValidationError::InvalidDiskUsage));
    }

    #[test]
    fn rejects_duplicate_or_empty_mount_points() {
        let mut duplicate = report();
        duplicate.disks[1].mount_point = "/".into();
        assert_eq!(duplicate.validate(), Err(ReportValidationError::DuplicateMountPoint));

        let mut empty = report();
        empty.disks[0].mount_point = " ".into();
        assert_eq!(empty.validate(), Err(ReportValidationError::InvalidMountPoint));
    }

    #[test]
    fn validates_each_wire_field_at_its_declared_boundary() {
        let mut input = report();
        input.node_id = "n".repeat(128);
        input.hostname = "h".repeat(253);
        input.agent_version = "v".repeat(64);
        input.cpu_percent = 0.0;
        input.memory = ByteUsage { used_bytes: i64::MAX as u64, total_bytes: i64::MAX as u64 };
        input.disks[0].usage =
            ByteUsage { used_bytes: i64::MAX as u64 - 1, total_bytes: i64::MAX as u64 };
        input.disks[1].mount_point = "m".repeat(256);
        input.validate().expect("all inclusive boundaries are valid");

        for (mutate, expected) in [
            (
                |r: &mut AgentReport| r.node_id = "n".repeat(129),
                ReportValidationError::InvalidNodeId,
            ),
            (
                |r: &mut AgentReport| r.hostname = "h".repeat(254),
                ReportValidationError::InvalidHostname,
            ),
            (
                |r: &mut AgentReport| r.agent_version = "v".repeat(65),
                ReportValidationError::InvalidAgentVersion,
            ),
            (
                |r: &mut AgentReport| r.cpu_percent = f64::INFINITY,
                ReportValidationError::InvalidCpuPercent,
            ),
            (|r: &mut AgentReport| r.cpu_percent = -0.1, ReportValidationError::InvalidCpuPercent),
            (
                |r: &mut AgentReport| r.memory.total_bytes = i64::MAX as u64 + 1,
                ReportValidationError::InvalidMemory,
            ),
            (
                |r: &mut AgentReport| r.disks[0].mount_point = "m".repeat(257),
                ReportValidationError::InvalidMountPoint,
            ),
        ] as [(fn(&mut AgentReport), ReportValidationError); 7]
        {
            let mut invalid = report();
            mutate(&mut invalid);
            assert_eq!(invalid.validate(), Err(expected));
        }
    }

    #[test]
    fn rejects_malformed_timestamp_and_accepts_empty_disk_list() {
        let mut empty = report();
        empty.disks.clear();
        empty.validate().expect("a node may have no visible disks");
        assert!(serde_json::from_str::<AgentReport>(
            r#"{"nodeId":"edge-a","bootId":"3f70ea4d-6e16-40ca-b78c-7cb54fc27d43","sequence":1,"hostname":"edge-a","agentVersion":"test","collectedAt":"not-a-timestamp","cpuPercent":1,"memory":{"usedBytes":1,"totalBytes":2},"disks":[]}"#,
        )
        .is_err());
    }
}
