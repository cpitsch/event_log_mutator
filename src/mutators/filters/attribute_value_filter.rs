use chrono::{DateTime, FixedOffset};
use regex::Regex;
use std::fmt::Display;

use process_mining::EventLog;

use crate::{
    mutation::{LogMutator, MutationResult},
    parsing::traits::DirName,
    utils::attributes::{
        get_bool_by_key, get_float_by_key, get_int_by_key, get_string_by_key, get_time_by_key,
        HasAttributes,
    },
};

#[derive(serde::Deserialize, Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(tag = "method", content = "value")]
pub enum AttributeFilterMethod {
    IntGreater(i64),
    IntGeq(i64),
    IntLess(i64),
    IntLeq(i64),
    IntEq(i64),
    /// Int attribute must be in range: low <= x < high
    IntRange(i64, i64),

    FloatGreater(f64),
    FloatGeq(f64),
    FloatLess(f64),
    FloatLeq(f64),
    FloatEq(f64),
    /// Float attribute must be in range: low <= x < high
    FloatRange(f64, f64),

    StringEq(String),
    // TODO: Made this string because it needs to impl Deserialize and PartialEq
    // However, this means that the Regex needs to be built every time we filter..
    StringRegex(String),

    BoolTrue,
    BoolFalse,

    // TODO: Why not FixedOffset (Actually I think it's because i use the old process_mining
    // version that still uses Utc)
    DateBefore(DateTime<FixedOffset>),
    DateAfter(DateTime<FixedOffset>),
    /// Date attribute must be in range: low <= x < high
    DateBetween(DateTime<FixedOffset>, DateTime<FixedOffset>),
}

impl Display for AttributeFilterMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::IntGreater(x) => format!("IntGreater_{x}"),
                Self::IntGeq(x) => format!("IntGeq_{x}"),
                Self::IntLess(x) => format!("IntLess_{x}"),
                Self::IntLeq(x) => format!("IntLeq_{x}"),
                Self::IntEq(x) => format!("IntEq_{x}"),
                Self::IntRange(start, end) => format!("IntRange_{start}_{end}"),

                Self::FloatGreater(x) => format!("FloatGreater_{x}"),
                Self::FloatGeq(x) => format!("FloatGeq_{x}"),
                Self::FloatLess(x) => format!("FloatLess_{x}"),
                Self::FloatLeq(x) => format!("FloatLeq_{x}"),
                Self::FloatEq(x) => format!("FloatEq_{x}"),
                Self::FloatRange(start, end) => format!("FloatRange_{start}_{end}"),

                Self::StringEq(s) => format!("StringEq_{s}"),
                // Leave out the regex because it might contain special characters
                Self::StringRegex(_) => "StringRegex".to_string(),
                // Self::StringRegex(re) => format!("StringRegex_{re}"),
                Self::BoolTrue => "IsTrue".to_string(),
                Self::BoolFalse => "IsFalse".to_string(),

                Self::DateBefore(d) => format!("DateBefore_{}", d),
                Self::DateAfter(d) => format!("DateAfter_{}", d),
                Self::DateBetween(d_start, d_end) => format!("DateBetween_{d_start}_{d_end}"),
            }
        )
    }
}

impl AttributeFilterMethod {
    fn apply(&self, item: &impl HasAttributes, key: &str) -> bool {
        match self {
            Self::IntGreater(x) => get_int_by_key(item, key).map(|val| &val > x),
            Self::IntGeq(x) => get_int_by_key(item, key).map(|val| &val >= x),
            Self::IntLess(x) => get_int_by_key(item, key).map(|val| &val < x),
            Self::IntLeq(x) => get_int_by_key(item, key).map(|val| &val <= x),
            Self::IntEq(x) => get_int_by_key(item, key).map(|val| &val == x),
            Self::IntRange(start, end) => {
                get_int_by_key(item, key).map(|val| (start..end).contains(&&val))
            }
            Self::FloatGreater(x) => get_float_by_key(item, key).map(|val| &val > x),
            Self::FloatGeq(x) => get_float_by_key(item, key).map(|val| &val >= x),
            Self::FloatLess(x) => get_float_by_key(item, key).map(|val| &val < x),
            Self::FloatLeq(x) => get_float_by_key(item, key).map(|val| &val <= x),
            Self::FloatEq(x) => get_float_by_key(item, key).map(|val| &val == x),
            Self::FloatRange(start, end) => {
                get_float_by_key(item, key).map(|val| (start..end).contains(&&val))
            }
            Self::StringEq(s) => get_string_by_key(item, key).map(|val| &val == s),
            Self::StringRegex(re) => get_string_by_key(item, key)
                .map(|val| Regex::new(re.as_str()).unwrap().is_match(&val)),

            Self::BoolTrue => get_bool_by_key(item, key),
            Self::BoolFalse => get_bool_by_key(item, key).map(|val| !val),

            Self::DateBefore(d) => get_time_by_key(item, key).map(|val| &val < d),
            Self::DateAfter(d) => get_time_by_key(item, key).map(|val| &val > d),
            Self::DateBetween(d_start, d_end) => {
                get_time_by_key(item, key).map(|val| d_start <= &val && &val <= d_end)
            }
        }
        // TODO: Could map_or_else to create MissingAttributeError, and handle it in
        // keep_trace/keep_event
        .unwrap_or(false)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum AttributeFilterTarget {
    /// Filter on trace-level attributes
    Trace,
    /// Keep only the events that match the filter, discarding empty traces
    // TODO: Keep empty cases?
    Event,
    /// Keep only the traces where at least one event matches the filter (and keep the entire trace)
    EventRequired,
    /// Remove all traces where at least one event matches the filter
    EventForbidden,
    /// Keep only traces where the filter matches on _all_ events.
    AllEvents,
    /// Keep only traces where the filter holds for the first event in the trace
    FirstEvent,
    /// Keep only traces where the filter holds for the last event in the trace
    LastEvent,
}

impl Display for AttributeFilterTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Trace => "Trace",
                Self::Event => "Event",
                Self::EventRequired => "EventRequired",
                Self::EventForbidden => "EventForbidden",
                Self::AllEvents => "AllEvents",
                Self::FirstEvent => "FirstEvent",
                Self::LastEvent => "LastEvent",
            }
        )
    }
}

#[derive(DirName)]
pub struct AttributeFilter {
    target: AttributeFilterTarget,
    // Could pose an issue for creating pathname? E.g., ":"
    key: String,
    filter_method: AttributeFilterMethod,
}

impl AttributeFilter {
    pub fn new(
        target: AttributeFilterTarget,
        key: impl Into<String>,
        filter_method: AttributeFilterMethod,
    ) -> Self {
        Self {
            target,
            key: key.into(),
            filter_method,
        }
    }

    fn keep(&self, item: &impl HasAttributes) -> bool {
        self.filter_method.apply(item, &self.key)
    }
}

impl LogMutator for AttributeFilter {
    fn apply_mut(&mut self, log: &mut EventLog) -> MutationResult<()> {
        match self.target {
            AttributeFilterTarget::Trace => log.traces.retain(|trace| self.keep(trace)),
            AttributeFilterTarget::EventRequired => log
                .traces
                .retain(|trace| trace.events.iter().any(|evt| self.keep(evt))),
            AttributeFilterTarget::EventForbidden => log
                .traces
                .retain(|trace| trace.events.iter().all(|evt| !self.keep(evt))),
            AttributeFilterTarget::Event => log.traces.retain_mut(|trace| {
                // Remove non-matching events
                trace.events.retain(|evt| self.keep(evt));
                // Remove empty traces
                !trace.events.is_empty()
            }),
            AttributeFilterTarget::AllEvents => log
                .traces
                .retain(|trace| trace.events.iter().all(|evt| self.keep(evt))),
            AttributeFilterTarget::FirstEvent => log.traces.retain(|trace| {
                trace
                    .events
                    .first()
                    .map(|evt| self.keep(evt))
                    // If the trace is empty, the filter does _not_ hold for the first event
                    .unwrap_or_default()
            }),
            AttributeFilterTarget::LastEvent => log.traces.retain(|trace| {
                trace
                    .events
                    .last()
                    .map(|evt| self.keep(evt))
                    // If the trace is empty, the filter does _not_ hold for the last event
                    .unwrap_or_default()
            }),
        }
        Ok(())
    }
}
