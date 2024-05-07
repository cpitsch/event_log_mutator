use itertools::Itertools;
use process_mining::{event_log::AttributeValue, EventLog};
use rand::seq::SliceRandom;

use crate::{mutation::LogMutator, utils::set_traceid_key};

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
        new_log.traces = log
            .traces
            .choose_multiple(rng, self.size)
            .cloned()
            .collect_vec();

        new_log
            .traces
            .iter_mut()
            .enumerate()
            .for_each(|(idx, trace)| {
                set_traceid_key(trace, AttributeValue::String((idx + 1).to_string())).unwrap();
            });
        new_log
    }
}
