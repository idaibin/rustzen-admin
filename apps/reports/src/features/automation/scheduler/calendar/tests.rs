use chrono::{DateTime, Utc};
use chrono_tz::America::New_York;

use super::{
    DueDecision, ScheduleZone, decide_due_occurrence, latest_due_occurrence, next_due_for_timezone,
    occurrence_is_effective,
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
