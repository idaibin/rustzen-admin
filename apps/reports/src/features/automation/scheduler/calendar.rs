use std::str::FromStr;

use chrono::{
    DateTime, Datelike, FixedOffset, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta,
    TimeZone, Utc,
};
use chrono_tz::Tz;

use crate::{common::error::AppError, features::automation::types::ScheduleRow};

const SCHEDULE_LATE_WINDOW_SECONDS: i64 = 60;

#[derive(Debug, Clone, Copy)]
pub(super) enum ScheduleZone {
    Iana(Tz),
    Fixed(FixedOffset),
}

#[derive(Debug, Clone)]
pub(super) struct DueOccurrence {
    pub(super) occurrence_key: String,
    pub(super) due_local: String,
    pub(super) due_at: Option<String>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum DueDecision {
    Enqueue,
    SkipMissed,
    SkipDstGap,
}

pub(crate) fn next_due_at(
    schedule: &ScheduleRow,
    now: DateTime<Utc>,
    timezone: &str,
) -> Result<Option<String>, AppError> {
    let zone = parse_schedule_timezone(timezone)?;
    next_due_occurrence(now, schedule, &zone).map(|value| value.and_then(|item| item.due_at))
}

fn next_due_occurrence(
    now: DateTime<Utc>,
    schedule: &ScheduleRow,
    timezone: &ScheduleZone,
) -> Result<Option<DueOccurrence>, AppError> {
    let due_time = parse_due_time(&schedule.due_time)?;
    match timezone {
        ScheduleZone::Iana(zone) => next_due_for_timezone(now, schedule, due_time, zone),
        ScheduleZone::Fixed(zone) => next_due_for_timezone(now, schedule, due_time, zone),
    }
}

pub(super) fn latest_due_occurrence(
    now: DateTime<Utc>,
    schedule: &ScheduleRow,
    timezone: &ScheduleZone,
) -> Result<Option<DueOccurrence>, AppError> {
    let due_time = parse_due_time(&schedule.due_time)?;
    match timezone {
        ScheduleZone::Iana(zone) => latest_due_for_timezone(now, schedule, due_time, zone),
        ScheduleZone::Fixed(zone) => latest_due_for_timezone(now, schedule, due_time, zone),
    }
}

fn next_due_for_timezone<T: TimeZone>(
    now: DateTime<Utc>,
    schedule: &ScheduleRow,
    due_time: NaiveTime,
    timezone: &T,
) -> Result<Option<DueOccurrence>, AppError> {
    let local_now = now.with_timezone(timezone);
    for day_offset in 0..=8 {
        let date = local_now.date_naive() + TimeDelta::days(day_offset);
        if !date_matches_schedule(date, schedule)? {
            continue;
        }
        let local = date.and_time(due_time);
        match timezone.from_local_datetime(&local) {
            LocalResult::Single(value) => {
                let due_at = value.with_timezone(&Utc);
                if due_at > now {
                    return Ok(Some(due_occurrence(local, Some(due_at))));
                }
            }
            LocalResult::Ambiguous(earlier, _) => {
                let due_at = earlier.with_timezone(&Utc);
                if due_at > now {
                    return Ok(Some(due_occurrence(local, Some(due_at))));
                }
            }
            LocalResult::None => continue,
        }
    }
    Ok(None)
}

fn latest_due_for_timezone<T: TimeZone>(
    now: DateTime<Utc>,
    schedule: &ScheduleRow,
    due_time: NaiveTime,
    timezone: &T,
) -> Result<Option<DueOccurrence>, AppError> {
    let local_now = now.with_timezone(timezone);
    for day_offset in 0..=8 {
        let date = local_now.date_naive() - TimeDelta::days(day_offset);
        if !date_matches_schedule(date, schedule)? {
            continue;
        }
        let local = date.and_time(due_time);
        let is_today = date == local_now.date_naive();
        let occurrence = match timezone.from_local_datetime(&local) {
            LocalResult::Single(value) => {
                let due_at = value.with_timezone(&Utc);
                if is_today && due_at > now {
                    continue;
                }
                due_occurrence(local, Some(due_at))
            }
            // The first offset is the earlier fold occurrence mandated by the product contract.
            LocalResult::Ambiguous(earlier, _) => {
                let due_at = earlier.with_timezone(&Utc);
                if is_today && due_at > now {
                    continue;
                }
                due_occurrence(local, Some(due_at))
            }
            LocalResult::None => {
                // A gap has no UTC instant; use the wall clock to decide whether
                // the nonexistent slot has already passed today.
                if is_today && local_now.time() < due_time {
                    continue;
                }
                due_occurrence(local, None)
            }
        };
        return Ok(Some(occurrence));
    }
    Ok(None)
}

fn due_occurrence(local: NaiveDateTime, due_at: Option<DateTime<Utc>>) -> DueOccurrence {
    let due_local = local.format("%Y-%m-%dT%H:%M").to_string();
    DueOccurrence {
        occurrence_key: due_local.clone(),
        due_local,
        due_at: due_at.map(|value| value.to_rfc3339()),
    }
}

pub(super) fn occurrence_is_effective(
    occurrence: &DueOccurrence,
    effective_at: &str,
    timezone: &ScheduleZone,
) -> bool {
    let Ok(effective_at) = DateTime::parse_from_rfc3339(effective_at) else {
        return false;
    };
    let effective_at = effective_at.with_timezone(&Utc);
    if let Some(due_at) = occurrence.due_at.as_deref() {
        let Ok(due_at) = DateTime::parse_from_rfc3339(due_at) else {
            return false;
        };
        return due_at.with_timezone(&Utc) >= effective_at;
    }
    // A DST gap has no UTC instant. Compare its local wall time against the
    // effective instant converted to the schedule's installation timezone.
    let effective_local = match timezone {
        ScheduleZone::Iana(zone) => effective_at.with_timezone(zone).naive_local(),
        ScheduleZone::Fixed(zone) => effective_at.with_timezone(zone).naive_local(),
    };
    let Ok(due_local) = NaiveDateTime::parse_from_str(&occurrence.due_local, "%Y-%m-%dT%H:%M")
    else {
        return false;
    };
    due_local >= effective_local
}

pub(super) fn decide_due_occurrence(
    now: DateTime<Utc>,
    occurrence: &DueOccurrence,
) -> Result<DueDecision, AppError> {
    if occurrence.due_at.is_none() {
        return Ok(DueDecision::SkipDstGap);
    }
    let due_at = DateTime::parse_from_rfc3339(occurrence.due_at.as_deref().expect("checked above"))
        .map_err(AppError::internal)?
        .with_timezone(&Utc);
    let lateness = now.signed_duration_since(due_at);
    if lateness < TimeDelta::zero() {
        return Err(AppError::internal("schedule occurrence is in the future"));
    }
    if lateness <= TimeDelta::seconds(SCHEDULE_LATE_WINDOW_SECONDS) {
        Ok(DueDecision::Enqueue)
    } else {
        Ok(DueDecision::SkipMissed)
    }
}

fn date_matches_schedule(date: NaiveDate, schedule: &ScheduleRow) -> Result<bool, AppError> {
    match schedule.cadence.as_str() {
        "daily" => {
            if schedule.weekday.is_some() {
                return Err(AppError::InvalidInput("daily schedule must not set weekday".into()));
            }
            Ok(true)
        }
        "weekly" => {
            let weekday = schedule
                .weekday
                .ok_or_else(|| AppError::InvalidInput("weekly schedule requires weekday".into()))?;
            if weekday > 6 {
                return Err(AppError::InvalidInput("weekday must be between 0 and 6".into()));
            }
            Ok(date.weekday().num_days_from_monday() as i64 == weekday)
        }
        _ => Err(AppError::InvalidInput("unsupported schedule cadence".into())),
    }
}

fn parse_due_time(value: &str) -> Result<NaiveTime, AppError> {
    NaiveTime::parse_from_str(value, "%H:%M")
        .map_err(|_| AppError::InvalidInput("dueTime must use HH:MM (24-hour) format".into()))
}

pub(super) fn parse_schedule_timezone(value: &str) -> Result<ScheduleZone, AppError> {
    let trimmed = value.trim();
    if let Ok(zone) = Tz::from_str(trimmed) {
        return Ok(ScheduleZone::Iana(zone));
    }
    parse_fixed_timezone(trimmed)
        .map(ScheduleZone::Fixed)
        .ok_or_else(|| AppError::InvalidInput(format!("invalid installation timezone: {trimmed}")))
}

fn parse_fixed_timezone(value: &str) -> Option<FixedOffset> {
    let seconds = match value {
        "UTC" | "Etc/UTC" | "Z" | "+00:00" | "-00:00" => 0,
        "Asia/Shanghai" | "Asia/Chongqing" | "Asia/Harbin" | "Asia/Urumqi" | "CST" => 8 * 3600,
        _ => {
            let sign = match value.as_bytes().first()? {
                b'+' => 1,
                b'-' => -1,
                _ => return None,
            };
            let rest = &value[1..];
            let (hours, minutes) = if let Some((hours, minutes)) = rest.split_once(':') {
                (hours.parse::<i32>().ok()?, minutes.parse::<i32>().ok()?)
            } else {
                (rest.parse::<i32>().ok()?, 0)
            };
            if !(0..=23).contains(&hours) || !(0..=59).contains(&minutes) {
                return None;
            }
            sign * ((hours * 3600) + (minutes * 60))
        }
    };
    FixedOffset::east_opt(seconds)
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use chrono_tz::America::New_York;

    use super::{
        DueDecision, ScheduleZone, decide_due_occurrence, latest_due_occurrence,
        next_due_for_timezone, occurrence_is_effective,
    };
    use crate::features::automation::types::ScheduleRow;

    fn schedule(cadence: &str, weekday: Option<i64>, due_time: &str) -> ScheduleRow {
        ScheduleRow {
            id: "schedule".into(),
            flow_id: "flow".into(),
            cadence: cadence.into(),
            weekday,
            due_time: due_time.into(),
            input_json: "{}".into(),
            description: String::new(),
            enabled: true,
            effective_at: "2026-08-10T00:00:00+00:00".into(),
            revision: 0,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn utc(value: &str) -> DateTime<Utc> {
        value.parse().expect("RFC3339 timestamp")
    }

    #[test]
    fn daily_occurrences_enqueue_inside_the_sixty_second_window_and_skip_late_slots() {
        let schedule = schedule("daily", None, "10:00");
        let zone = ScheduleZone::Iana("UTC".parse().expect("UTC timezone"));
        let occurrence = latest_due_occurrence(utc("2026-08-10T10:00:30Z"), &schedule, &zone)
            .expect("candidate")
            .expect("daily candidate");
        assert_eq!(
            decide_due_occurrence(utc("2026-08-10T10:00:30Z"), &occurrence).expect("decision"),
            DueDecision::Enqueue
        );
        assert_eq!(
            decide_due_occurrence(utc("2026-08-10T10:02:00Z"), &occurrence).expect("decision"),
            DueDecision::SkipMissed
        );
        assert_eq!(
            decide_due_occurrence(utc("2026-08-10T10:01:00.001Z"), &occurrence)
                .expect("fractional decision"),
            DueDecision::SkipMissed
        );
    }

    #[test]
    fn weekly_occurrence_uses_iso_weekday_and_next_due_skips_a_dst_gap() {
        // Monday is 0 in the schedule contract.
        let weekly = schedule("weekly", Some(0), "10:00");
        let zone = ScheduleZone::Iana("UTC".parse().expect("UTC timezone"));
        let occurrence = latest_due_occurrence(utc("2026-08-10T10:00:00Z"), &weekly, &zone)
            .expect("candidate")
            .expect("weekly candidate");
        assert_eq!(occurrence.occurrence_key, "2026-08-10T10:00");

        let dst_schedule = schedule("daily", None, "02:30");
        let zone = ScheduleZone::Iana(New_York);
        let next = next_due_for_timezone(
            utc("2026-03-09T12:00:00Z"),
            &dst_schedule,
            chrono::NaiveTime::from_hms_opt(2, 30, 0).expect("time"),
            match &zone {
                ScheduleZone::Iana(zone) => zone,
                ScheduleZone::Fixed(_) => unreachable!(),
            },
        )
        .expect("next due")
        .expect("valid post-gap occurrence");
        assert_eq!(next.due_at.as_deref(), Some("2026-03-10T06:30:00+00:00"));
    }

    #[test]
    fn dst_gap_is_skipped_and_fold_resolves_to_earlier_offset() {
        let gap_schedule = schedule("daily", None, "02:30");
        let zone = ScheduleZone::Iana(New_York);
        let gap = latest_due_occurrence(utc("2026-03-08T08:00:00Z"), &gap_schedule, &zone)
            .expect("candidate")
            .expect("gap candidate");
        assert!(gap.due_at.is_none());
        assert_eq!(
            decide_due_occurrence(utc("2026-03-08T08:00:00Z"), &gap).expect("decision"),
            DueDecision::SkipDstGap
        );

        let fold_schedule = schedule("daily", None, "01:30");
        let fold = latest_due_occurrence(utc("2026-11-01T12:00:00Z"), &fold_schedule, &zone)
            .expect("candidate")
            .expect("fold candidate");
        assert_eq!(fold.occurrence_key, "2026-11-01T01:30");
        assert_eq!(fold.due_at.as_deref(), Some("2026-11-01T05:30:00+00:00"));
        let second_fold = latest_due_occurrence(utc("2026-11-01T06:15:00Z"), &fold_schedule, &zone)
            .expect("second fold candidate")
            .expect("second fold occurrence");
        assert_eq!(second_fold.occurrence_key, fold.occurrence_key);
        assert_eq!(
            decide_due_occurrence(utc("2026-11-01T06:15:00Z"), &second_fold)
                .expect("second fold decision"),
            DueDecision::SkipMissed
        );
        let first_fold = latest_due_occurrence(utc("2026-11-01T05:30:30Z"), &fold_schedule, &zone)
            .expect("first fold candidate")
            .expect("first fold occurrence");
        assert_eq!(
            decide_due_occurrence(utc("2026-11-01T05:30:30Z"), &first_fold)
                .expect("first fold decision"),
            DueDecision::Enqueue
        );
    }

    #[test]
    fn occurrence_before_effective_at_is_not_replayed_after_create_or_reenable() {
        let mut schedule = schedule("daily", None, "10:00");
        let zone = ScheduleZone::Iana("UTC".parse().expect("UTC timezone"));
        let occurrence = latest_due_occurrence(utc("2026-08-10T10:00:30Z"), &schedule, &zone)
            .expect("candidate")
            .expect("same-day candidate");

        schedule.effective_at = "2026-08-10T10:00:30+00:00".into();
        assert!(!occurrence_is_effective(&occurrence, &schedule.effective_at, &zone));

        schedule.effective_at = "2026-08-10T09:59:59+00:00".into();
        assert!(occurrence_is_effective(&occurrence, &schedule.effective_at, &zone));
    }
}
