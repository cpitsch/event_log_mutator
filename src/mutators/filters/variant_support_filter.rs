use itertools::Itertools;
use process_mining::{event_log::Trace, EventLog};

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
    pub fn new(num_supporting_cases: impl Into<usize>) -> VariantSupportFilter {
        VariantSupportFilter {
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
        .map(get_activity_label)
        .collect::<AttributeResult<Vec<_>>>()
        .map_err(|e| MutationError::MissingAttributeError("VariantSupportFilter", e))
}
