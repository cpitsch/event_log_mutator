use itertools::Itertools;
use process_mining::core::event_data::case_centric::{EventLog, Trace};

use crate::{
    mutation::{LogMutator, MutationError, MutationResult},
    parsing::traits::DirName,
    utils::attributes::{get_activity_label, AttributeResult},
};

/// Mutation to retain only the cases whose variant (projection on the executed
/// activity) occurs frequently enough.
#[derive(DirName)]
pub struct VariantSupportFilter {
    /// The threshold to use for variant filtering. A variant must occur at least
    /// this many times to not be removed from the event log.
    #[dirname(rename = "thresh", no_split)]
    num_supporting_cases: usize,
}

impl VariantSupportFilter {
    pub fn new(num_supporting_cases: impl Into<usize>) -> Self {
        Self {
            num_supporting_cases: num_supporting_cases.into(),
        }
    }
}

impl LogMutator for VariantSupportFilter {
    fn apply_mut(&mut self, log: &mut EventLog) -> MutationResult<()> {
        let variants = log
            .traces
            .iter()
            .map(get_variant)
            .collect::<MutationResult<Vec<_>>>()?;
        let variant_counts = variants.iter().counts();

        let mut keep_trace = variants.iter().map(|trace_variant| {
            *variant_counts.get(trace_variant).unwrap_or(&0) >= self.num_supporting_cases
        });

        log.traces.retain(|_| keep_trace.next().unwrap());

        Ok(())
    }
}

fn get_variant(trace: &Trace) -> MutationResult<Vec<String>> {
    trace
        .events
        .iter()
        .map(|evt| get_activity_label(evt).cloned())
        .collect::<AttributeResult<Vec<_>>>()
        .map_err(|e| MutationError::AttributeError("VariantSupportFilter", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use process_mining::event_log;

    #[test]
    fn filter_at_least() {
        let mut filter = VariantSupportFilter::new(3usize);
        let log = event_log!(
            // abcd 2 times
            ["a", "b", "c", "d"],
            ["a", "b", "c", "d"],
            // acbd 3 times
            ["a", "c", "b", "d"],
            ["a", "c", "b", "d"],
            ["a", "c", "b", "d"],
            // ac 4 times
            ["a", "c"],
            ["a", "c"],
            ["a", "c"],
            ["a", "c"],
        );
        assert_eq!(
            event_log!(
                ["a", "c", "b", "d"] { "concept:name" => 2 },
                ["a", "c", "b", "d"] { "concept:name" => 3 },
                ["a", "c", "b", "d"] { "concept:name" => 4 },
                ["a", "c"] { "concept:name" => 5 },
                ["a", "c"] { "concept:name" => 6 },
                ["a", "c"] { "concept:name" => 7 },
                ["a", "c"] { "concept:name" => 8 },
            ),
            filter.apply(&log).unwrap()
        );
    }
}
