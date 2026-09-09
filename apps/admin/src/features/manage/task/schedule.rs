use std::time::Duration;

use chrono::FixedOffset;

use crate::common::error::ServiceError;

pub(super) fn parse_fixed_timezone(value: &str) -> Result<FixedOffset, ServiceError> {
    let trimmed = value.trim();
    let seconds = match trimmed {
        "UTC" | "Etc/UTC" | "Z" | "+00:00" | "-00:00" => 0,
        "Asia/Shanghai" | "Asia/Chongqing" | "Asia/Harbin" | "Asia/Urumqi" | "CST" => 8 * 3600,
        _ => parse_timezone_offset_seconds(trimmed).ok_or_else(|| {
            ServiceError::InvalidOperation(format!(
                "Invalid RUSTZEN_TIMEZONE: {trimmed}; use UTC, Asia/Shanghai, or offsets like +08:00"
            ))
        })?,
    };
    FixedOffset::east_opt(seconds).ok_or_else(|| {
        ServiceError::InvalidOperation(format!("Invalid RUSTZEN_TIMEZONE offset: {trimmed}"))
    })
}

fn parse_timezone_offset_seconds(value: &str) -> Option<i32> {
    let sign = match value.as_bytes().first()? {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let rest = &value[1..];
    let (hours, minutes): (i32, i32) = match rest.split_once(':') {
        Some((hours, minutes)) => (hours.parse().ok()?, minutes.parse().ok()?),
        None => (rest.parse().ok()?, 0),
    };
    if !(0..=23).contains(&hours) || !(0..=59).contains(&minutes) {
        return None;
    }
    Some(sign * ((hours * 3600) + (minutes * 60)))
}

pub(super) fn task_run_timeout_duration(timeout_secs: u64) -> Duration {
    Duration::from_secs(timeout_secs.max(1))
}
