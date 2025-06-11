use std::collections::HashSet;

use chrono::{DateTime, FixedOffset, TimeDelta};

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

#[derive(Debug, PartialEq, Clone)]
pub enum AttributeLevel {
    Event,
    Trace,
    Log,
    Unknown,
}

impl std::fmt::Display for AttributeLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let level_str = match self {
            Self::Event => "Event",
            Self::Trace => "Trace",
            Self::Log => "Log",
            Self::Unknown => "Unkown",
        };
        write!(f, "{}", level_str)
    }
}

#[derive(Error, Debug, PartialEq, Clone, Eq)]
pub enum AttributeErrorKind {
    #[error("not found")]
    MissingAttribute,
    #[error("has unexpected type. Expected {0}, found {1:?}")]
    TypeMismatch(String, AttributeValue),
}

#[derive(Error, Debug, Clone, PartialEq)]
#[error("{level}-level attribute \"{key}\" {kind}.")]
pub struct AttributeError {
    pub level: AttributeLevel,
    pub key: String,
    pub kind: AttributeErrorKind,
}

impl AttributeError {
    pub fn missing_attribute(key: impl Into<String>) -> Self {
        Self {
            level: AttributeLevel::Unknown,
            key: key.into(),
            kind: AttributeErrorKind::MissingAttribute,
        }
    }

    pub fn type_mismatch(
        key: impl Into<String>,
        expected: impl Into<String>,
        found: AttributeValue,
    ) -> Self {
        Self {
            level: AttributeLevel::Unknown,
            key: key.into(),
            kind: AttributeErrorKind::TypeMismatch(expected.into(), found),
        }
    }

    pub fn with_level(mut self, level: AttributeLevel) -> Self {
        self.level = level;
        self
    }
}

pub type AttributeResult<T> = Result<T, AttributeError>;

fn get_attribute_value(from: &impl HasAttributes, key: &str) -> AttributeResult<AttributeValue> {
    from.get_attributes()
        .get_by_key(key)
        .ok_or_else(|| AttributeError::missing_attribute(key))
        .map(|v| v.value.clone())
}

pub fn get_string_by_key(from: &impl HasAttributes, key: &str) -> AttributeResult<String> {
    get_attribute_value(from, key).map(|value| {
        value
            .try_as_string()
            .cloned()
            .ok_or_else(|| AttributeError::type_mismatch(key, "String", value))
    })?
}

pub fn get_time_by_key(
    from: &impl HasAttributes,
    key: &str,
) -> AttributeResult<DateTime<FixedOffset>> {
    get_attribute_value(from, key).map(|value| {
        value
            .try_as_date()
            .cloned()
            .ok_or_else(|| AttributeError::type_mismatch(key, "DateTime", value))
    })?
}

pub fn get_int_by_key(from: &impl HasAttributes, key: &str) -> AttributeResult<i64> {
    get_attribute_value(from, key).map(|value| {
        value
            .try_as_int()
            .cloned()
            .ok_or_else(|| AttributeError::type_mismatch(key, "Int", value))
    })?
}

pub fn get_float_by_key(from: &impl HasAttributes, key: &str) -> AttributeResult<f64> {
    get_attribute_value(from, key).map(|value| {
        value
            .try_as_float()
            .cloned()
            .ok_or_else(|| AttributeError::type_mismatch(key, "Float", value))
    })?
}

pub fn get_bool_by_key(from: &impl HasAttributes, key: &str) -> AttributeResult<bool> {
    get_attribute_value(from, key).map(|value| {
        value
            .try_as_bool()
            .cloned()
            .ok_or_else(|| AttributeError::type_mismatch(key, "Bool", value))
    })?
}

pub fn get_activity_label(event: &Event) -> AttributeResult<String> {
    get_string_by_key(event, ACTIVITY_KEY).map_err(|e| e.with_level(AttributeLevel::Event))
}

pub fn get_start_timestamp(event: &Event) -> AttributeResult<DateTime<FixedOffset>> {
    get_time_by_key(event, START_TIMESTAMP_KEY).map_err(|e| e.with_level(AttributeLevel::Event))
}

pub fn get_complete_timestamp(event: &Event) -> AttributeResult<DateTime<FixedOffset>> {
    get_time_by_key(event, TIMESTAMP_KEY).map_err(|e| e.with_level(AttributeLevel::Event))
}

pub fn get_traceid(trace: &Trace) -> AttributeResult<String> {
    get_string_by_key(trace, TRACEID_KEY).map_err(|e| e.with_level(AttributeLevel::Event))
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
    to: DateTime<FixedOffset>,
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
    if trace.events.is_empty() {
        return Ok(HashSet::new());
    }

    let activity_timestamp_pairs = trace
        .events
        .iter()
        .map(
            |event| -> AttributeResult<(String, DateTime<FixedOffset>)> {
                let act = get_activity_label(event)?;
                let complete = get_complete_timestamp(event)?;
                Ok((act, complete))
            },
        )
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
    if trace.events.is_empty() {
        return Ok(HashSet::new());
    }

    // TODO: Could this just be get_start_activities on the reversed trace?
    let activity_timestamp_pairs = trace
        .events
        .iter()
        .map(
            |event| -> AttributeResult<(String, DateTime<FixedOffset>)> {
                let act = get_activity_label(event)?;
                let complete = get_complete_timestamp(event)?;
                Ok((act, complete))
            },
        )
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
