use chrono::TimeDelta;
use process_mining::event_log::{AttributeValue, Event};

use crate::{
    constants::{NO_COMPLETE_TIMESTAMP_MSG, NO_START_TIMESTAMP_MSG},
    mutation::EventMutator,
    utils::{get_service_time, get_start_timestamp, set_complete_timestamp},
};

pub struct ServiceTimeMultiplier {
    factor: f32,
}

impl ServiceTimeMultiplier {
    pub fn new(factor: f32) -> Self {
        Self { factor }
    }
}

/// Try to multiply nanoseconds by the float, then round ns and convert back to TimeDelta.
/// If TimeDelta spans more than 292.5 yrs (2^63ns), work on seconds instead.
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
    }
}
