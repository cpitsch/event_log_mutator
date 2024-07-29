use itertools::Itertools;
use process_mining::event_log::Trace;

use crate::{mutation::LogMutator, parsing::dir_name_trait::DirName, utils::get_activity_label};

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
    fn apply(&self, log: &process_mining::EventLog) -> process_mining::EventLog {
        let mut new_log = log.clone();
        let variant_counts = log.traces.iter().map(get_variant).counts();

        new_log.traces.retain(|trace| {
            let variant = get_variant(trace);
            let count = variant_counts.get(&variant).unwrap_or(&0);
            *count >= self.num_supporting_cases
        });

        new_log
    }
}

fn get_variant(trace: &Trace) -> Vec<String> {
    trace
        .events
        .iter()
        .map(|evt| get_activity_label(evt).unwrap())
        .collect()
}
