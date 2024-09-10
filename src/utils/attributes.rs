use std::collections::HashSet;

use chrono::{DateTime, TimeDelta, Utc};
use itertools::Itertools;
use process_mining::{
    event_log::{AttributeValue, Attributes, Event, Trace, XESEditableAttribute},
    EventLog,
};

use crate::constants::{
    ACTIVITY_KEY, NO_ACTIVITY_LABEL_MSG, NO_COMPLETE_TIMESTAMP_MSG, NO_START_TIMESTAMP_MSG,
    NO_TRACEID_MSG, START_TIMESTAMP_KEY, TIMESTAMP_KEY, TRACEID_KEY,
};
pub trait HasAttributes {
    fn get_attributes(&self) -> &Attributes;
}

impl HasAttributes for Trace {
    fn get_attributes(&self) -> &Attributes {
        &self.attributes
    }
}

impl HasAttributes for Event {
    fn get_attributes(&self) -> &Attributes {
        &self.attributes
    }
}

pub fn get_string_by_key(from: &impl HasAttributes, key: &str) -> Option<String> {
    from.get_attributes()
        .get_by_key(key)?
        .value
        .try_as_string()
        .cloned()
}

pub fn get_time_by_key(from: &impl HasAttributes, key: &str) -> Option<DateTime<Utc>> {
    from.get_attributes()
        .get_by_key(key)?
        .value
        .try_as_date()
        .cloned()
}

pub fn get_activity_label(event: &Event) -> Option<String> {
    get_string_by_key(event, ACTIVITY_KEY)
}

pub fn get_start_timestamp(event: &Event) -> Option<DateTime<Utc>> {
    get_time_by_key(event, START_TIMESTAMP_KEY)
}

pub fn get_complete_timestamp(event: &Event) -> Option<DateTime<Utc>> {
    get_time_by_key(event, TIMESTAMP_KEY)
}

pub fn get_traceid(trace: &Trace) -> Option<String> {
    get_string_by_key(trace, TRACEID_KEY)
}

pub fn get_service_time(event: &Event) -> Option<chrono::TimeDelta> {
    let start = get_start_timestamp(event)?;
    let end = get_complete_timestamp(event)?;
    Some(end - start)
}

pub fn set_trace_attribute_by_key(trace: &mut Trace, key: &'static str, value: AttributeValue) {
    if let Some(attr) = trace.attributes.get_by_key_mut(key) {
        attr.value = value;
    } else {
        trace.attributes.add_to_attributes(key.to_owned(), value);
    }
}

pub fn set_event_attribute_by_key(event: &mut Event, key: &'static str, value: AttributeValue) {
    if let Some(attr) = event.attributes.get_by_key_mut(key) {
        attr.value = value;
    } else {
        event.attributes.add_to_attributes(key.to_owned(), value);
    }
}

pub fn set_activity_label(event: &mut Event, value: AttributeValue) {
    set_event_attribute_by_key(event, ACTIVITY_KEY, value)
}

pub fn set_complete_timestamp(event: &mut Event, value: AttributeValue) {
    set_event_attribute_by_key(event, TIMESTAMP_KEY, value)
}
pub fn set_start_timestamp(event: &mut Event, value: AttributeValue) {
    set_event_attribute_by_key(event, START_TIMESTAMP_KEY, value)
}

pub fn set_traceid(trace: &mut Trace, value: AttributeValue) {
    set_trace_attribute_by_key(trace, TRACEID_KEY, value)
}

/// Shift the start- and complete timestamp of all  events starting at index
///`from` by the TimeDelta `by`.
pub fn shift_events_by(trace: &mut Trace, by: TimeDelta, from: usize) {
    trace.events.iter_mut().skip(from).for_each(|evt| {
        // Shift the events start and end by the timedelta
        let new_start_timestamp = get_start_timestamp(evt).expect(NO_START_TIMESTAMP_MSG) + by;
        let new_complete_timestamp =
            get_complete_timestamp(evt).expect(NO_COMPLETE_TIMESTAMP_MSG) + by;

        set_start_timestamp(evt, AttributeValue::Date(new_start_timestamp));
        set_complete_timestamp(evt, AttributeValue::Date(new_complete_timestamp));
    })
}

pub fn change_event_duration(trace: &mut Trace, index: usize, to: DateTime<Utc>) {
    let evt = trace.events.get_mut(index).unwrap();
    let old_complete_timestamp = get_complete_timestamp(evt).unwrap();
    let timestamp_difference = to - old_complete_timestamp;
    set_complete_timestamp(evt, AttributeValue::Date(to));

    shift_events_by(trace, timestamp_difference, index + 1);
}

pub fn get_traceids(log: &EventLog) -> HashSet<String> {
    log.traces
        .iter()
        .map(|trace| get_traceid(trace).expect(NO_TRACEID_MSG))
        .collect()
}

pub fn get_activities(log: &EventLog) -> HashSet<String> {
    log.traces
        .iter()
        .flat_map(|trace| {
            trace
                .events
                .iter()
                .map(|event| get_activity_label(event).expect(NO_ACTIVITY_LABEL_MSG))
                .collect::<HashSet<String>>()
        })
        .collect()
}

pub fn get_start_activities(trace: &Trace) -> HashSet<String> {
    let activity_timestamp_pairs = trace
        .events
        .iter()
        .map(|event| {
            (
                get_activity_label(event).expect(NO_ACTIVITY_LABEL_MSG),
                get_complete_timestamp(event).expect(NO_COMPLETE_TIMESTAMP_MSG),
            )
        })
        .collect_vec();

    // Errors if the vec is empty
    // Should  never be empty as the trace is not empty either
    let earliest_timestamp = activity_timestamp_pairs
        .iter()
        .min_by_key(|(_, time)| time)
        .unwrap()
        .1;
    activity_timestamp_pairs
        .iter()
        .filter(|(_, time)| *time <= earliest_timestamp)
        .map(|(activity, _)| activity)
        .cloned()
        .collect()
}

pub fn get_end_activities(trace: &Trace) -> HashSet<String> {
    // TODO: Could this just be get_start_activities on the reversed trace?
    let activity_timestamp_pairs = trace
        .events
        .iter()
        .map(|event| {
            (
                get_activity_label(event).expect(NO_ACTIVITY_LABEL_MSG),
                get_complete_timestamp(event).expect(NO_COMPLETE_TIMESTAMP_MSG),
            )
        })
        .collect_vec();

    // Errors if the vec is empty
    // Should never be empty as the trace is not empty either
    let latest_timestamp = activity_timestamp_pairs
        .iter()
        .max_by_key(|(_, time)| time)
        .unwrap()
        .1;
    activity_timestamp_pairs
        .iter()
        .filter(|(_, time)| *time >= latest_timestamp)
        .map(|(activity, _)| activity)
        .cloned()
        .collect()
}
