use std::collections::HashSet;

use chrono::{DateTime, TimeDelta, Utc};

use process_mining::{
    event_log::{AttributeValue, Attributes, Event, Trace, XESEditableAttribute},
    EventLog,
};
use thiserror::Error;

use crate::constants::{ACTIVITY_KEY, START_TIMESTAMP_KEY, TIMESTAMP_KEY, TRACEID_KEY};
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

#[derive(Debug, PartialEq, PartialOrd)]
pub enum AttributeLevel {
    Event,
    Trace,
    Log,
}

impl std::fmt::Display for AttributeLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let level_str = match self {
            AttributeLevel::Event => "event",
            AttributeLevel::Trace => "trace",
            AttributeLevel::Log => "log",
        };
        write!(f, "{}", level_str)
    }
}

#[derive(Error, Debug, PartialEq, PartialOrd)]
#[error("Missing {level}-level attribute {key}")]
pub struct MissingAttributeError {
    pub level: AttributeLevel,
    pub key: &'static str,
}

pub type AttributeResult<T> = Result<T, MissingAttributeError>;

impl MissingAttributeError {
    pub fn new(level: AttributeLevel, key: &'static str) -> Self {
        Self { level, key }
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

pub fn get_activity_label(event: &Event) -> AttributeResult<String> {
    get_string_by_key(event, ACTIVITY_KEY)
        .ok_or_else(|| MissingAttributeError::new(AttributeLevel::Event, ACTIVITY_KEY))
}

pub fn get_start_timestamp(event: &Event) -> AttributeResult<DateTime<Utc>> {
    get_time_by_key(event, START_TIMESTAMP_KEY)
        .ok_or_else(|| MissingAttributeError::new(AttributeLevel::Event, START_TIMESTAMP_KEY))
}

pub fn get_complete_timestamp(event: &Event) -> AttributeResult<DateTime<Utc>> {
    get_time_by_key(event, TIMESTAMP_KEY)
        .ok_or_else(|| MissingAttributeError::new(AttributeLevel::Event, TIMESTAMP_KEY))
}

pub fn get_traceid(trace: &Trace) -> AttributeResult<String> {
    get_string_by_key(trace, TRACEID_KEY)
        .ok_or_else(|| MissingAttributeError::new(AttributeLevel::Trace, TRACEID_KEY))
}

pub fn get_service_time(event: &Event) -> AttributeResult<chrono::TimeDelta> {
    let start = get_start_timestamp(event)?;
    let end = get_complete_timestamp(event)?;
    Ok(end - start)
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
pub fn shift_events_by(trace: &mut Trace, by: TimeDelta, from: usize) -> AttributeResult<()> {
    for evt in trace.events.iter_mut().skip(from) {
        // Shift the events start and end by the timedelta
        let new_start_timestamp = get_start_timestamp(evt)? + by;
        let new_complete_timestamp = get_complete_timestamp(evt)? + by;

        set_start_timestamp(evt, AttributeValue::Date(new_start_timestamp));
        set_complete_timestamp(evt, AttributeValue::Date(new_complete_timestamp));
    }
    Ok(())
}

pub fn change_event_duration(
    trace: &mut Trace,
    index: usize,
    to: DateTime<Utc>,
) -> AttributeResult<()> {
    let evt = trace.events.get_mut(index).unwrap();
    let old_complete_timestamp = get_complete_timestamp(evt)?;
    let timestamp_difference = to - old_complete_timestamp;
    set_complete_timestamp(evt, AttributeValue::Date(to));

    shift_events_by(trace, timestamp_difference, index + 1)?;
    Ok(())
}

pub fn get_traceids(log: &EventLog) -> AttributeResult<HashSet<String>> {
    log.traces.iter().map(get_traceid).collect()
}

pub fn get_activities(log: &EventLog) -> AttributeResult<HashSet<String>> {
    log.traces
        .iter()
        .flat_map(|trace| trace.events.iter().map(get_activity_label))
        .collect::<AttributeResult<HashSet<String>>>()
}

pub fn get_start_activities(trace: &Trace) -> AttributeResult<HashSet<String>> {
    let activity_timestamp_pairs = trace
        .events
        .iter()
        .map(|event| -> AttributeResult<(String, DateTime<Utc>)> {
            let act = get_activity_label(event)?;
            let complete = get_complete_timestamp(event)?;
            Ok((act, complete))
        })
        .collect::<AttributeResult<Vec<_>>>()?;

    // Errors if the vec is empty
    // Should  never be empty as the trace is not empty either
    let earliest_timestamp = activity_timestamp_pairs
        .iter()
        .min_by_key(|(_, time)| time)
        .unwrap()
        .1;
    Ok(activity_timestamp_pairs
        .into_iter()
        // TODO: Could do take_while instead if we assume that the trace is sorted
        .filter(|(_, time)| *time <= earliest_timestamp)
        .map(|(activity, _)| activity)
        .collect())
}

pub fn get_end_activities(trace: &Trace) -> AttributeResult<HashSet<String>> {
    // TODO: Could this just be get_start_activities on the reversed trace?
    let activity_timestamp_pairs = trace
        .events
        .iter()
        .map(|event| -> AttributeResult<(String, DateTime<Utc>)> {
            let act = get_activity_label(event)?;
            let complete = get_complete_timestamp(event)?;
            Ok((act, complete))
        })
        .collect::<AttributeResult<Vec<_>>>()?;

    // Errors if the vec is empty
    // Should never be empty as the trace is not empty either
    let latest_timestamp = activity_timestamp_pairs
        .iter()
        .max_by_key(|(_, time)| time)
        .unwrap()
        .1;
    Ok(activity_timestamp_pairs
        .into_iter()
        .filter(|(_, time)| *time >= latest_timestamp)
        .map(|(activity, _)| activity)
        .collect())
}
