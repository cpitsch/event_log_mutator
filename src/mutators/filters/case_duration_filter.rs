use std::fmt::Display;

use chrono::TimeDelta;
use process_mining::{event_log::Trace, EventLog};
use serde::Deserialize;

use crate::{
    mutation::{LogMutator, MutationError, MutationResult},
    parsing::traits::DirName,
    utils::{
        attributes::{get_complete_timestamp, AttributeResult},
        errors::retain_err,
    },
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
    threshold: TimeDelta,
}

impl CaseDurationFilter {
    pub fn new(threshold: TimeDelta) -> Self {
        CaseDurationFilter {
            threshold,
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

    fn keep_trace(&self, trace: &Trace) -> AttributeResult<bool> {
        // Could theoretically use first() and last(), but I don't know for
        // _certain_ that the trace is ordered correctly (?)
        let complete_timestamps = trace
            .events
            .iter()
            .map(get_complete_timestamp)
            .collect::<AttributeResult<Vec<_>>>()?;

        let earliest_timestamp = *complete_timestamps.iter().min().unwrap();
        let latest_timestamp = complete_timestamps.into_iter().max().unwrap();

        let duration = latest_timestamp - earliest_timestamp;

        Ok(match self.sense {
            ComparisonSense::Less => duration < self.threshold,
            ComparisonSense::LEQ => duration <= self.threshold,
            ComparisonSense::GEQ => duration >= self.threshold,
            ComparisonSense::Greater => duration > self.threshold,
        })
    }
}

impl LogMutator for CaseDurationFilter {
    fn apply_mut(&mut self, log: &mut EventLog) -> MutationResult<()> {
        retain_err(&mut log.traces, |trace| self.keep_trace(trace))
            .map_err(|e| MutationError::MissingAttributeError("CaseDurationFilter", e))?;
        Ok(())
    }
}
