use chrono::TimeDelta;
use process_mining::event_log::{AttributeValue, Event};
use rand::random;

use crate::{
    constants::{NO_ACTIVITY_LABEL_MSG, NO_COMPLETE_TIMESTAMP_MSG, NO_START_TIMESTAMP_MSG},
    mutation::EventMutator,
    utils::{get_activity_label, get_service_time, get_start_timestamp, set_complete_timestamp},
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

    pub fn for_activity(mut self, activity: String) -> Self {
        self.activity = Some(activity);
        self
    }

    pub fn with_probability(mut self, probability: f32) -> Self {
        self.probability = probability;
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

impl EventMutator for ServiceTimeMultiplier {
    fn apply(&self, evt: &Event) -> Event {
        if self.should_mutate(evt) {
            let mut new_event = evt.clone();
            let start_timestamp = get_start_timestamp(&new_event).expect(NO_START_TIMESTAMP_MSG);
            let service_time = get_service_time(&new_event).expect(NO_COMPLETE_TIMESTAMP_MSG);
            let new_serivce_time = multiply_timedelta_by_float(service_time, &self.factor);

            set_complete_timestamp(
                &mut new_event,
                AttributeValue::Date(start_timestamp + new_serivce_time),
            )
            .expect_err("Error setting completion timestamp");
            new_event
        } else {
            evt.clone()
        }
    }
}
