use chrono::TimeDelta;
use process_mining::core::event_data::case_centric::{EventLog, Trace};

use crate::{
    mutation::{LogMutator, MutationError, MutationResult},
    mutators::filters::ComparisonSense,
    parsing::traits::DirName,
    utils::{
        attributes::{get_complete_timestamp, AttributeResult},
        errors::retain_err,
    },
};

#[derive(DirName)]
pub struct CaseDurationFilter {
    sense: ComparisonSense,
    threshold: TimeDelta,
}

impl CaseDurationFilter {
    pub fn new(threshold: TimeDelta) -> Self {
        Self {
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

        // TODO: Could be smart about this and compute them together
        let earliest_timestamp = complete_timestamps.iter().min().copied();
        let latest_timestamp = complete_timestamps.into_iter().max();

        let duration = latest_timestamp
            .map(|dur| *dur - earliest_timestamp.unwrap())
            .unwrap_or_default();

        Ok(self.sense.compare(&duration, &self.threshold))
    }
}

impl LogMutator for CaseDurationFilter {
    fn apply_mut(&mut self, log: &mut EventLog) -> MutationResult<()> {
        retain_err(&mut log.traces, |trace| self.keep_trace(trace))
            .map_err(|e| MutationError::AttributeError("CaseDurationFilter", e))?;
        Ok(())
    }
}
