use chrono::{SubsecRound, TimeDelta};
use process_mining::event_log::{AttributeValue, Event, Trace};
use rand::random;

use crate::{
    constants::{NO_ACTIVITY_LABEL_MSG, NO_COMPLETE_TIMESTAMP_MSG, NO_START_TIMESTAMP_MSG},
    mutation::TraceMutator,
    utils::{
        get_activity_label, get_complete_timestamp, get_service_time, get_start_timestamp,
        set_complete_timestamp, shift_events_by,
    },
};

/// Mutation to increase the service time by a factor.
pub struct ServiceTimeMultiplier {
    /// Only mutate events with this activity. Defaults to all activities (None).
    /// Use [`ServiceTimeMultiplier::for_activity`] to for a specific activity.
    activity: Option<String>,
    /// The probability to apply the mutation to a matching event. Ranges from 0 to 1.
    /// Use [`ServiceTimeMultiplier::with_probability`] for a specific probability.
    probability: f32,
    /// The factor to multiply the service time by.
    factor: f32,
}

impl ServiceTimeMultiplier {
    pub fn new(factor: f32) -> Self {
        Self {
            activity: None,
            probability: 1.0,
            factor,
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

    fn apply_event(&self, evt: &Event) -> Event {
        if self.should_mutate(evt) {
            let mut new_event = evt.clone();
            let start_timestamp = get_start_timestamp(&new_event).expect(NO_START_TIMESTAMP_MSG);
            let service_time = get_service_time(&new_event).expect(NO_COMPLETE_TIMESTAMP_MSG);
            let new_serivce_time = multiply_timedelta_by_float(service_time, &self.factor);

            set_complete_timestamp(
                &mut new_event,
                // Round duration seconds to 6 decimal places so pm4py imports it correctly
                AttributeValue::Date((start_timestamp + new_serivce_time).round_subsecs(6)),
            )
            .unwrap();
            new_event
        } else {
            evt.clone()
        }
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
    fn apply(&self, trace: &Trace) -> Trace {
        // new_trace
        //     .events
        //     .iter_mut()
        //     .for_each(|event| *event = self.apply_event(event));

        let mut new_trace = trace.clone();
        for i in 0..new_trace.events.len() {
            let event = new_trace.events.get_mut(i).unwrap();
            if self.should_mutate(event) {
                let old_complete_timestamp =
                    get_complete_timestamp(event).expect(NO_COMPLETE_TIMESTAMP_MSG);
                *event = self.apply_event(event);
                let shifted_by = get_complete_timestamp(event).expect(NO_COMPLETE_TIMESTAMP_MSG)
                    - old_complete_timestamp;

                // Need to move all following events if changed service time
                // So: For each following event, move its start and completion timestamp
                // by the amount of time added to this service time
                // Otherwise, we induce control-flow changes that are unwanted side-effects
                // from this mutation
                shift_events_by(&mut new_trace, shifted_by, i + 1);
            }
        }

        // TODO: No longer needed because timestamps are updated
        // Sort events by complete timestamp because we changed those only for some
        // new_trace
        //     .events
        //     .sort_by_key(|evt| get_complete_timestamp(evt).expect(NO_COMPLETE_TIMESTAMP_MSG));
        //
        new_trace
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, TimeZone, Utc};
    use process_mining::event_log::Attribute;

    use super::*;

    fn new_event(
        activity: impl Into<String>,
        start_timestamp: DateTime<Utc>,
        service_time: TimeDelta,
    ) -> Event {
        Event {
            attributes: vec![
                Attribute::new(
                    "concept:name".to_owned(),
                    AttributeValue::String(activity.into()),
                ),
                Attribute::new(
                    "start_timestamp".to_owned(),
                    AttributeValue::Date(start_timestamp),
                ),
                Attribute::new(
                    "time:timestamp".to_owned(),
                    AttributeValue::Date(start_timestamp + service_time),
                ),
            ],
        }
    }

    fn test_trace() -> Trace {
        let date = Utc
            .with_ymd_and_hms(2024, 4, 29, 1, 0, 0)
            .earliest()
            .unwrap();
        Trace {
            attributes: Vec::default(),
            events: vec![
                new_event("a", date, TimeDelta::hours(1)),
                // Starts exactly as the previous finishes.
                new_event("b", date + TimeDelta::hours(1), TimeDelta::hours(2)),
            ],
        }
    }

    fn get_control_flow(trace: &Trace) -> Vec<String> {
        trace
            .events
            .iter()
            .map(|evt| get_activity_label(evt).unwrap())
            .collect()
    }

    #[test]
    fn does_not_affect_control_flow() {
        let trace = test_trace();

        let new_trace = ServiceTimeMultiplier::new(100.0)
            .for_activity("a")
            .apply(&trace);

        assert!(get_control_flow(&trace) == get_control_flow(&new_trace));
    }

    #[test]
    fn default_affects_all_activities() {
        let trace = test_trace();
        let new_trace = ServiceTimeMultiplier::new(100.0).apply(&trace);

        assert!(trace
            .events
            .iter()
            .zip(new_trace.events.iter())
            .all(|(e1, e2)| { get_service_time(e1) < get_service_time(e2) }));
    }

    #[test]
    fn only_affects_for_activity() {
        let trace = test_trace();
        let new_trace = ServiceTimeMultiplier::new(100.0)
            .for_activity("a")
            .apply(&trace);

        assert!(trace
            .events
            .iter()
            .zip(new_trace.events.iter())
            .all(|(e1, e2)| {
                // Assumes control flow isnt affected, which is tested by [`does_not_affect_control_flow`]
                assert!(get_activity_label(e1).unwrap() == get_activity_label(e2).unwrap());

                if get_activity_label(e1).unwrap() == "a" {
                    get_service_time(e1) < get_service_time(e2)
                } else {
                    get_service_time(e1) == get_service_time(e2)
                }
            }));
    }

    #[test]
    fn zero_probability_does_nothing() {
        let trace = test_trace();
        let new_trace = ServiceTimeMultiplier::new(100.0)
            .with_probability(0.0)
            .apply(&trace);

        assert!(trace
            .events
            .iter()
            .map(|evt| get_service_time(evt).unwrap())
            .eq(new_trace
                .events
                .iter()
                .map(|evt| get_service_time(evt).unwrap())));
    }
}
