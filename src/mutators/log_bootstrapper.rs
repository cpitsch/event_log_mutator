use process_mining::{event_log::AttributeValue, EventLog};
use rand::seq::SliceRandom;

use crate::{mutation::LogMutator, utils::set_traceid};

/// Mutator to create a new log by randomly sampling cases with replacement.
/// The sampled cases are assigned unique case ids ("0" ... "`size`").
pub struct LogBootstrapper {
    /// The number of cases to sample.
    size: usize,
}

impl LogBootstrapper {
    pub fn new(size: usize) -> Self {
        Self { size }
    }
}

impl LogMutator for LogBootstrapper {
    fn apply(&self, log: &EventLog) -> EventLog {
        let mut new_log = log.clone();
        // Sample `output_size` random cases
        let rng = &mut rand::thread_rng();
        new_log.traces = Vec::with_capacity(self.size);

        for i in 0..self.size {
            let mut new_trace = log
                .traces
                .choose(rng)
                .expect("Cannot bootstrap an empty event log.")
                .clone();

            set_traceid(&mut new_trace, AttributeValue::String(i.to_string()));

            new_log.traces.push(new_trace);
        }

        new_log
    }
}
