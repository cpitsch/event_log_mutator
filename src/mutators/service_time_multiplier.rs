use chrono::{SubsecRound, TimeDelta};
use process_mining::event_log::{Event, Trace};
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    constants::{NO_ACTIVITY_LABEL_MSG, NO_START_TIMESTAMP_MSG},
    mutation::TraceMutator,
    parsing::dir_name_trait::DirName,
    utils::{change_event_duration, get_activity_label, get_service_time, get_start_timestamp},
};

/// Mutation to increase the service time by a factor.
#[derive(DirName)]
pub struct ServiceTimeMultiplier {
    /// Only mutate events with this activity. Defaults to all activities (None).
    /// Use [`ServiceTimeMultiplier::for_activity`] to for a specific activity.
    #[dirname(rename = "")]
    activity: Option<String>,
    /// The probability to apply the mutation to a matching event. Ranges from 0 to 1.
    /// Use [`ServiceTimeMultiplier::with_probability`] for a specific probability.
    #[dirname(rename = "p", no_split)]
    probability: f32,
    /// The factor to multiply the service time by.
    #[dirname(rename = "x", no_split)]
    factor: f32,
    /// Optional seed for the random number generator. Ensures reproducible results
    /// across runs.
    seed: Option<u64>,
    #[dirname(ignore)]
    rng: StdRng,
}

impl ServiceTimeMultiplier {
    pub fn new(factor: f32) -> Self {
        Self {
            activity: None,
            probability: 1.0,
            factor,
            seed: None,
            rng: StdRng::from_entropy(),
        }
    }

    fn should_mutate(&mut self, event: &Event) -> bool {
        (
            // Check that the event matches the requirements
            self.activity.clone().map_or(true, |act| {
                get_activity_label(event).expect(NO_ACTIVITY_LABEL_MSG) == act
            })
        ) && (
            // Check mutation probability
            self.rng.gen::<f32>() < self.probability
        )
    }

    pub fn for_activity(mut self, activity: impl Into<String>) -> Self {
        self.activity = Some(activity.into());
        self
    }

    pub fn with_probability(mut self, probability: f32) -> Self {
        self.probability = probability;
        self
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self.rng = StdRng::seed_from_u64(seed);
        self
    }
}

/// Helper function to multiply a timedelta by a float.
///
/// - Try to multiply nanoseconds by the float, then round ns and convert back to TimeDelta.
/// - If TimeDelta spans more than 2^63 nanoseconds (292.5 yrs), work on seconds instead.
fn multiply_timedelta_by_float(timedelta: TimeDelta, factor: &f32) -> TimeDelta {
    // Can't multiply TimeDelta by float, so need to be a bit less exact

    // num_nanoseconds only returns None if Timedelta > ~295.2yrs
    if let Some(ns) = timedelta.num_nanoseconds() {
        let new_ns = (ns as f32) * *factor;
        TimeDelta::nanoseconds(new_ns.round() as i64)
    } else {
        let seconds = timedelta.num_seconds() as f32;
        let new_seconds = seconds * *factor;
        TimeDelta::seconds(new_seconds.round() as i64)
    }
}

impl TraceMutator for ServiceTimeMultiplier {
    fn apply(&mut self, trace: &Trace) -> Trace {
        let mut new_trace = trace.clone();
        for i in 0..new_trace.events.len() {
            let event = new_trace.events.get_mut(i).unwrap();
            if self.should_mutate(event) {
                let start_timestamp = get_start_timestamp(event).expect(NO_START_TIMESTAMP_MSG);
                let service_time = get_service_time(event).expect(NO_START_TIMESTAMP_MSG);
                let new_service_time = multiply_timedelta_by_float(service_time, &self.factor);
                change_event_duration(
                    &mut new_trace,
                    i,
                    (start_timestamp + new_service_time).round_subsecs(6),
                );
            }
        }

        new_trace
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_fixtures::{abcd_trace, get_control_flow};
    use itertools::izip;
    use rstest::rstest;

    #[rstest]
    #[case::factor_1(1.0, 3600)]
    #[case::float_factor(1.5, 5400)]
    #[case::factor_0(0.0, 0)]
    fn timedelta_multiplication(#[case] factor: f32, #[case] expected_seconds: i64) {
        let delta = TimeDelta::hours(1);
        let multiplied_delta = multiply_timedelta_by_float(delta, &factor);

        assert_eq!(
            multiplied_delta.num_seconds()
                // Round due to floating point imprecision
                + if multiplied_delta.subsec_nanos() >= 500_000_000 {
                    1
                } else {
                    0
                },
            expected_seconds
        )
    }

    #[rstest]
    fn does_not_affect_control_flow(abcd_trace: Trace) {
        let new_trace = ServiceTimeMultiplier::new(100.0)
            .for_activity("a")
            .apply(&abcd_trace);

        assert_eq!(get_control_flow(&abcd_trace), get_control_flow(&new_trace));
    }

    #[rstest]
    fn default_affects_all_activities(abcd_trace: Trace) {
        let new_trace = ServiceTimeMultiplier::new(100.0).apply(&abcd_trace);

        assert!(abcd_trace
            .events
            .iter()
            .zip(new_trace.events.iter())
            .all(|(e1, e2)| { get_service_time(e1) < get_service_time(e2) }));
    }

    #[rstest]
    fn only_affects_for_activity(abcd_trace: Trace) {
        let new_trace = ServiceTimeMultiplier::new(100.0)
            .for_activity("a")
            .apply(&abcd_trace);

        assert!(abcd_trace
            .events
            .iter()
            .zip(new_trace.events.iter())
            .all(|(e1, e2)| {
                // Assumes control flow isnt affected, which is tested by [`does_not_affect_control_flow`]
                assert!(get_activity_label(e1).unwrap() == get_activity_label(e2).unwrap());

                if get_activity_label(e1).unwrap() == "a" {
                    get_service_time(e1) < get_service_time(e2)
                } else {
                    // Service time is unchanged
                    get_service_time(e1) == get_service_time(e2)
                }
            }));
    }

    #[rstest]
    fn zero_probability_does_nothing(abcd_trace: Trace) {
        let new_trace = ServiceTimeMultiplier::new(100.0)
            .with_probability(0.0)
            .apply(&abcd_trace);

        assert!(abcd_trace
            .events
            .iter()
            .map(|evt| get_service_time(evt).unwrap())
            .eq(new_trace
                .events
                .iter()
                .map(|evt| get_service_time(evt).unwrap())));
    }
    #[rstest]
    fn affects_times_correctly(abcd_trace: Trace) {
        let durations: Vec<_> = abcd_trace
            .events
            .iter()
            .map(|event| get_service_time(event).unwrap())
            .collect();

        let start_timestamps: Vec<_> = abcd_trace
            .events
            .iter()
            .map(|event| get_start_timestamp(event).unwrap())
            .collect();

        let factor = 100.0;
        let new_durations: Vec<_> = ServiceTimeMultiplier::new(factor)
            .for_activity("a")
            .with_probability(1.0)
            .apply(&abcd_trace)
            .events
            .iter()
            .map(|event| get_service_time(event).unwrap())
            .collect();

        izip!(
            get_control_flow(&abcd_trace),
            start_timestamps,
            durations,
            new_durations
        )
        .for_each(|(act, start, old_dur, new_dur)| {
            if act == *"a" {
                // Activity a is incremented by 1 day
                // Currently fails due to rounding the completion timestamp to 6 digits
                // So the service time isnt exactly this..
                let expected_completion_timestamp =
                    (start + multiply_timedelta_by_float(old_dur, &factor)).round_subsecs(6);
                let expected_dur = expected_completion_timestamp - start;

                assert_eq!(new_dur, expected_dur);
            } else {
                // All others are left untouched
                assert_eq!(new_dur, old_dur)
            }
        });
    }

    #[rstest]
    fn seeded_gives_same_result(abcd_trace: Trace) {
        let new_trace_1 = ServiceTimeMultiplier::new(2.0)
            .for_activity("a")
            .with_probability(0.5)
            .with_seed(42)
            .apply(&abcd_trace);

        let new_trace_2 = ServiceTimeMultiplier::new(2.0)
            .for_activity("a")
            .with_probability(0.5)
            .with_seed(42)
            .apply(&abcd_trace);

        let trace_1_service_times: Vec<_> = new_trace_1
            .events
            .iter()
            .map(|evt| get_service_time(evt).unwrap())
            .collect();

        let trace_2_service_times: Vec<_> = new_trace_2
            .events
            .iter()
            .map(|evt| get_service_time(evt).unwrap())
            .collect();

        assert_eq!(trace_1_service_times, trace_2_service_times);
    }
}
