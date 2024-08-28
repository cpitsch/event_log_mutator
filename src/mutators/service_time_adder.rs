use chrono::TimeDelta;
use process_mining::event_log::Event;
use rand::random;

use crate::{
    constants::{NO_ACTIVITY_LABEL_MSG, NO_COMPLETE_TIMESTAMP_MSG},
    mutation::TraceMutator,
    parsing::dir_name_trait::DirName,
    utils::{change_event_duration, get_activity_label, get_complete_timestamp},
};

/// Mutation to increase the service time by a constant amount by increasing
/// the completion timestamp
#[derive(DirName)]
pub struct ServiceTimeAdder {
    /// Only apply the mutation to events with this activity. Defaults to all activities.
    /// Use [`ServiceTimeMutation::for_activity`] to set a specific activity.
    #[dirname(rename = "")]
    activity: Option<String>,
    /// The probability to apply the mutation to a matching event. Ranges from 0 to 1.
    /// Use [`ServiceTimeMutation::with_probability`] to set a probability.
    #[dirname(rename = "p", no_split)]
    probability: f32,
    /// The time difference to add to the service time.
    #[dirname(rename = "by")]
    timedelta: TimeDelta,
}

impl ServiceTimeAdder {
    pub fn new(delta: TimeDelta) -> Self {
        Self {
            activity: None,
            probability: 1.0,
            timedelta: delta,
        }
    }

    fn should_mutate(&self, event: &Event) -> bool {
        (
            // Check that the event matches the requirements
            self.activity.clone().map_or(true, |act| {
                get_activity_label(event).expect(NO_ACTIVITY_LABEL_MSG) == act
            })
        ) && (
            // Check mutation probability
            random::<f32>() < self.probability
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
}

impl TraceMutator for ServiceTimeAdder {
    fn apply(
        &mut self,
        trace: &process_mining::event_log::Trace,
    ) -> process_mining::event_log::Trace {
        let mut new_trace = trace.clone();
        for i in 0..new_trace.events.len() {
            let event = new_trace.events.get_mut(i).unwrap();
            if self.should_mutate(event) {
                let new_complete_timestamp = get_complete_timestamp(event)
                    .expect(NO_COMPLETE_TIMESTAMP_MSG)
                    + self.timedelta;

                change_event_duration(&mut new_trace, i, new_complete_timestamp);
            }
        }
        new_trace
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        test_fixtures::{abcd_trace, get_control_flow},
        utils::get_service_time,
    };
    use itertools::izip;
    use process_mining::event_log::Trace;
    use rstest::rstest;

    #[rstest]
    fn does_not_affect_control_flow(abcd_trace: Trace) {
        let new_trace = ServiceTimeAdder::new(TimeDelta::days(1))
            .for_activity("a")
            .apply(&abcd_trace);

        assert_eq!(get_control_flow(&abcd_trace), get_control_flow(&new_trace));
    }

    #[rstest]
    fn default_affects_all_activities(abcd_trace: Trace) {
        let new_trace = ServiceTimeAdder::new(TimeDelta::days(1)).apply(&abcd_trace);

        assert!(abcd_trace
            .events
            .iter()
            .zip(new_trace.events.iter())
            .all(|(e1, e2)| { get_service_time(e1) < get_service_time(e2) }));
    }

    #[rstest]
    fn only_affects_for_activity(abcd_trace: Trace) {
        let new_trace = ServiceTimeAdder::new(TimeDelta::days(1))
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
        let new_trace = ServiceTimeAdder::new(TimeDelta::days(1))
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

        let increment = TimeDelta::days(1);
        let new_durations: Vec<_> = ServiceTimeAdder::new(increment)
            .for_activity("a")
            .with_probability(1.0)
            .apply(&abcd_trace)
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
}
