use chrono::TimeDelta;
use process_mining::event_log::{Event, Trace};
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    mutation::{MutationError, MutationResult, TraceMutator},
    parsing::traits::DirName,
    utils::attributes::{change_event_duration, get_activity_label, get_complete_timestamp},
};

/// Mutation to increase the service time by a constant amount by increasing
/// the completion timestamp
#[derive(DirName)]
pub struct ServiceTimeAdder {
    /// Only apply the mutation to events with this activity. Defaults to all activities.
    /// Use [`ServiceTimeAdder::for_activity`] to set a specific activity.
    #[dirname(rename = "")]
    activity: Option<String>,
    /// The time difference to add to the service time.
    #[dirname(rename = "by")]
    timedelta: TimeDelta,
    /// The probability to apply the mutation to a matching event. Ranges from 0 to 1.
    /// Use [`ServiceTimeAdder::with_probability`] to set a probability.
    #[dirname(rename = "p", no_split)]
    probability: f32,
    /// Optional seed for the random number generator. Ensures reproducible results
    /// across runs. Use [`ServiceTimeAdder::with_seed`] to set the seed.
    seed: Option<u64>,
    #[dirname(ignore)]
    rng: StdRng,
}

impl ServiceTimeAdder {
    pub fn new(delta: TimeDelta) -> Self {
        Self {
            activity: None,
            probability: 1.0,
            timedelta: delta,
            seed: None,
            rng: StdRng::from_entropy(),
        }
    }

    fn should_mutate(&mut self, event: &Event) -> MutationResult<bool> {
        let activity = get_activity_label(event)
            .map_err(|e| MutationError::AttributeError("ServiceTimeAdder", e))?;
        let should_mutate = (
            // Check that the event matches the requirements
            self.activity.as_ref().map_or(true, |act| activity == act)
        ) && (
            // Check mutation probability
            self.rng.gen::<f32>() < self.probability
        );
        Ok(should_mutate)
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

impl TraceMutator for ServiceTimeAdder {
    fn apply_mut(&mut self, trace: &mut Trace) -> MutationResult<()> {
        for i in 0..trace.events.len() {
            let event = trace.events.get(i).unwrap();
            if self.should_mutate(event)? {
                let new_complete_timestamp = *get_complete_timestamp(event)
                    .map_err(|e| MutationError::AttributeError("ServiceTimeAdder", e))?
                    + self.timedelta;

                change_event_duration(trace, i, new_complete_timestamp)
                    .map_err(|e| MutationError::AttributeError("ServiceTimeAdder", e))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        test_fixtures::{abcd_trace, get_control_flow},
        utils::attributes::get_service_time,
    };
    use itertools::izip;
    use process_mining::event_log::Trace;
    use rstest::rstest;

    #[rstest]
    fn does_not_affect_control_flow(abcd_trace: Trace) {
        let new_trace = ServiceTimeAdder::new(TimeDelta::days(1))
            .for_activity("a")
            .apply(&abcd_trace)
            .unwrap();

        assert_eq!(get_control_flow(&abcd_trace), get_control_flow(&new_trace));
    }

    #[rstest]
    fn default_affects_all_activities(abcd_trace: Trace) {
        let new_trace = ServiceTimeAdder::new(TimeDelta::days(1))
            .apply(&abcd_trace)
            .unwrap();

        assert!(abcd_trace
            .events
            .iter()
            .zip(new_trace.events.iter())
            .all(|(e1, e2)| { get_service_time(e1).unwrap() < get_service_time(e2).unwrap() }));
    }

    #[rstest]
    fn only_affects_for_activity(abcd_trace: Trace) {
        let new_trace = ServiceTimeAdder::new(TimeDelta::days(1))
            .for_activity("a")
            .apply(&abcd_trace)
            .unwrap();

        assert!(abcd_trace
            .events
            .iter()
            .zip(new_trace.events.iter())
            .all(|(e1, e2)| {
                // Assumes control flow isnt affected, which is tested by [`does_not_affect_control_flow`]
                assert!(get_activity_label(e1).unwrap() == get_activity_label(e2).unwrap());

                if get_activity_label(e1).unwrap() == "a" {
                    get_service_time(e1).unwrap() < get_service_time(e2).unwrap()
                } else {
                    // Service time is unchanged
                    get_service_time(e1).unwrap() == get_service_time(e2).unwrap()
                }
            }));
    }

    #[rstest]
    fn zero_probability_does_nothing(abcd_trace: Trace) {
        let new_trace = ServiceTimeAdder::new(TimeDelta::days(1))
            .with_probability(0.0)
            .apply(&abcd_trace)
            .unwrap();

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

        let increment = TimeDelta::days(1);
        let new_durations: Vec<_> = ServiceTimeAdder::new(increment)
            .for_activity("a")
            .with_probability(1.0)
            .apply(&abcd_trace)
            .unwrap()
            .events
            .iter()
            .map(|event| get_service_time(event).unwrap())
            .collect();

        izip!(get_control_flow(&abcd_trace), durations, new_durations).for_each(
            |(act, old_dur, new_dur)| {
                if act == *"a" {
                    // Activity a is incremented by 1 day
                    assert_eq!(new_dur, old_dur + increment);
                } else {
                    // All others are left untouched
                    assert_eq!(new_dur, old_dur)
                }
            },
        );
    }

    #[rstest]
    fn seeded_gives_same_result(abcd_trace: Trace) {
        let increment = TimeDelta::days(1);
        let new_trace_1 = ServiceTimeAdder::new(increment)
            .for_activity("a")
            .with_probability(0.5)
            .with_seed(42)
            .apply(&abcd_trace)
            .unwrap();

        let new_trace_2 = ServiceTimeAdder::new(increment)
            .for_activity("a")
            .with_probability(0.5)
            .with_seed(42)
            .apply(&abcd_trace)
            .unwrap();

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
