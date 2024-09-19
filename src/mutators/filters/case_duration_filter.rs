use std::fmt::Display;

use process_mining::{event_log::Trace, EventLog};
use serde::Deserialize;

use crate::{
    constants::NO_COMPLETE_TIMESTAMP_MSG, mutation::LogMutator, parsing::dir_name_trait::DirName,
    utils::attributes::get_complete_timestamp,
};

#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum ComparisonSense {
    #[serde(alias = "less", alias = "<")]
    Less,
    #[serde(alias = "leq", alias = "<=")]
    LEQ,
    #[serde(alias = "geq", alias = ">=")]
    GEQ,
    #[serde(alias = "greater", alias = ">")]
    Greater,
}

impl Default for ComparisonSense {
    fn default() -> Self {
        Self::LEQ
    }
}

impl Display for ComparisonSense {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Less => "less",
                Self::LEQ => "leq",
                Self::GEQ => "geq",
                Self::Greater => "greater",
            }
        )
    }
}

#[derive(DirName)]
pub struct CaseDurationFilter {
    sense: ComparisonSense,
    years: f32,
    days: f32,
    hours: f32,
    minutes: f32,
    seconds: f32,
}

impl CaseDurationFilter {
    pub fn new(
        years: Option<f32>,
        days: Option<f32>,
        hours: Option<f32>,
        minutes: Option<f32>,
        seconds: Option<f32>,
    ) -> Self {
        CaseDurationFilter {
            years: years.unwrap_or(0.0),
            days: days.unwrap_or(0.0),
            hours: hours.unwrap_or(0.0),
            minutes: minutes.unwrap_or(0.0),
            seconds: seconds.unwrap_or(0.0),
            sense: ComparisonSense::default(),
        }
    }

    pub fn leq(mut self) -> Self {
        self.sense = ComparisonSense::LEQ;
        self
    }

    pub fn geq(mut self) -> Self {
        self.sense = ComparisonSense::GEQ;
        self
    }

    pub fn with_sense(mut self, sense: ComparisonSense) -> Self {
        self.sense = sense;
        self
    }

    fn keep_trace(&self, trace: &Trace, max_duration: &chrono::TimeDelta) -> bool {
        // Could theoretically use first() and last(), but I don't know for
        // _certain_ that the trace is ordered correctly
        let earliest_timestamp = trace
            .events
            .iter()
            .map(|event| get_complete_timestamp(event).expect(NO_COMPLETE_TIMESTAMP_MSG))
            .min()
            .unwrap();
        let latest_timestamp = trace
            .events
            .iter()
            .map(|event| get_complete_timestamp(event).expect(NO_COMPLETE_TIMESTAMP_MSG))
            .max()
            .unwrap();

        let duration = latest_timestamp - earliest_timestamp;

        match self.sense {
            ComparisonSense::Less => duration < *max_duration,
            ComparisonSense::LEQ => duration <= *max_duration,
            ComparisonSense::GEQ => duration >= *max_duration,
            ComparisonSense::Greater => duration > *max_duration,
        }
    }

    fn get_total_seconds(&self) -> i64 {
        let days = (365.0 * self.years) + self.days;
        let hours = (24.0 * days) + self.hours;
        let minutes = (60.0 * hours) + self.minutes;
        let seconds = (60.0 * minutes) + self.seconds;

        seconds as i64
    }
}

impl LogMutator for CaseDurationFilter {
    fn apply_mut(&mut self, log: &mut EventLog) {
        let max_duration = chrono::TimeDelta::seconds(self.get_total_seconds());
        log.traces
            .retain(|trace| self.keep_trace(trace, &max_duration));
    }
}
