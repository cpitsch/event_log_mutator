use process_mining::core::event_data::case_centric::{EventLog, Trace};

use crate::{
    mutation::{LogMutator, MutationError, MutationResult},
    mutators::filters::ComparisonSense,
    parsing::traits::DirName,
    utils::{attributes::AttributeResult, errors::retain_err},
};

#[derive(DirName)]
pub struct TraceLengthFilter {
    length: usize,
    sense: ComparisonSense,
}

impl TraceLengthFilter {
    pub fn new(length: usize) -> Self {
        Self {
            length,
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
        Ok(self.sense.compare(&trace.events.len(), &self.length))
    }
}

impl LogMutator for TraceLengthFilter {
    fn apply_mut(&mut self, log: &mut EventLog) -> MutationResult<()> {
        retain_err(&mut log.traces, |trace| self.keep_trace(trace))
            .map_err(|e| MutationError::AttributeError("TraceLengthFilter", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use process_mining::event_log;

    use super::*;

    #[test]
    fn leq() {
        let mut filter = TraceLengthFilter::new(2).leq();
        let log = event_log!(["a"], ["a", "b"], ["a", "b", "c"], ["a", "b", "c", "d"]);

        assert_eq!(2, filter.apply(&log).unwrap().traces.len());
    }
    #[test]
    fn geq() {
        let mut filter = TraceLengthFilter::new(2).geq();
        let log = event_log!(["a"], ["a", "b"], ["a", "b", "c"], ["a", "b", "c", "d"]);

        assert_eq!(3, filter.apply(&log).unwrap().traces.len());
    }
    #[test]
    fn less() {
        let mut filter = TraceLengthFilter::new(2).with_sense(ComparisonSense::Less);
        let log = event_log!(["a"], ["a", "b"], ["a", "b", "c"], ["a", "b", "c", "d"]);

        assert_eq!(1, filter.apply(&log).unwrap().traces.len());
    }
    #[test]
    fn greater() {
        let mut filter = TraceLengthFilter::new(2).with_sense(ComparisonSense::Greater);
        let log = event_log!(["a"], ["a", "b"], ["a", "b", "c"], ["a", "b", "c", "d"]);

        assert_eq!(2, filter.apply(&log).unwrap().traces.len());
    }

    #[test]
    fn empty_trace() {
        let log = event_log!([], ["a"], ["a", "b"]);
        let mut filter = TraceLengthFilter::new(1).with_sense(ComparisonSense::Less);

        assert_eq!(1, filter.apply(&log).unwrap().traces.len());
    }
}
