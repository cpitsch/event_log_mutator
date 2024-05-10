use std::{error::Error, fmt::Display};

use chrono::{DateTime, TimeDelta, Utc};
use process_mining::event_log::{AttributeValue, Event, Trace, XESEditableAttribute};

use crate::constants::{
    ACTIVITY_KEY, NO_COMPLETE_TIMESTAMP_MSG, NO_START_TIMESTAMP_MSG, START_TIMESTAMP_KEY,
    TIMESTAMP_KEY, TRACEID_KEY,
};

#[derive(Debug)]
pub struct WriteAttributeNotFoundError(&'static str);

impl Display for WriteAttributeNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Attempt to write non-existent attribute \"{}\"", self.0)
    }
}
impl Error for WriteAttributeNotFoundError {}

pub fn get_string_by_key(event: &Event, key: &str) -> Option<String> {
    event
        .attributes
        .get_by_key(key)?
        .value
        .try_as_string()
        .cloned()
}

pub fn get_time_by_key(event: &Event, key: &str) -> Option<DateTime<Utc>> {
    event
        .attributes
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

pub fn get_traceid(event: &Event) -> Option<String> {
    get_string_by_key(event, TRACEID_KEY)
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

pub fn set_traceid_key(event: &mut Trace, value: AttributeValue) {
    set_trace_attribute_by_key(event, TRACEID_KEY, value)
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
