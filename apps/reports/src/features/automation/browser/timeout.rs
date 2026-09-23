use std::time::Duration;

use super::super::types::FlowStep;

const PAUSE_TIMEOUT_MARGIN: Duration = Duration::from_secs(1);

pub(super) fn for_step(step: &FlowStep, default: Duration) -> Duration {
    match step {
        FlowStep::Pause { duration_ms } => {
            default.max(Duration::from_millis(*duration_ms).saturating_add(PAUSE_TIMEOUT_MARGIN))
        }
        _ => default,
    }
}

#[cfg(test)]
mod tests {
    use super::for_step;
    use crate::features::automation::types::FlowStep;
    use std::time::Duration;

    #[test]
    fn ordinary_steps_keep_the_default_timeout() {
        assert_eq!(
            for_step(&FlowStep::Click { selector: "#save".into() }, Duration::from_secs(3)),
            Duration::from_secs(3),
        );
    }

    #[test]
    fn pause_uses_the_larger_of_default_or_duration_with_margin() {
        assert_eq!(
            for_step(&FlowStep::Pause { duration_ms: 500 }, Duration::from_secs(3)),
            Duration::from_secs(3),
        );
        assert_eq!(
            for_step(&FlowStep::Pause { duration_ms: 5_000 }, Duration::from_secs(3)),
            Duration::from_secs(6),
        );
        assert_eq!(
            for_step(&FlowStep::Pause { duration_ms: 30_000 }, Duration::from_secs(1)),
            Duration::from_secs(31),
        );
    }
}
