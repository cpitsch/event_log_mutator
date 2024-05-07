use std::{error::Error, fmt::Display};

use chrono::{DateTime, Utc};
use process_mining::event_log::{AttributeValue, Event, XESEditableAttribute};

use crate::constants::{ACTIVITY_KEY, START_TIMESTAMP_KEY, TIMESTAMP_KEY};

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

pub fn get_service_time(event: &Event) -> Option<chrono::TimeDelta> {
    let start = get_start_timestamp(event)?;
    let end = get_complete_timestamp(event)?;
    Some(end - start)
}

pub fn set_activity_label(
    event: &mut Event,
    value: AttributeValue,
) -> Result<(), WriteAttributeNotFoundError> {
    event
        .attributes
        .get_by_key_mut(ACTIVITY_KEY)
        .ok_or(WriteAttributeNotFoundError(ACTIVITY_KEY))?
        .value = value;
    Ok(())
}

pub fn set_complete_timestamp(
    event: &mut Event,
    value: AttributeValue,
) -> Result<(), WriteAttributeNotFoundError> {
    event
        .attributes
        .get_by_key_mut(TIMESTAMP_KEY)
        .ok_or(WriteAttributeNotFoundError(TIMESTAMP_KEY))?
        .value = value;
    Ok(())
}
pub fn set_start_timestamp(
    event: &mut Event,
    value: AttributeValue,
) -> Result<(), WriteAttributeNotFoundError> {
    event
        .attributes
        .get_by_key_mut(START_TIMESTAMP_KEY)
        .ok_or(WriteAttributeNotFoundError(START_TIMESTAMP_KEY))?
        .value = value;
    Ok(())
}
