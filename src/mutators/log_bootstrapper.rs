use process_mining::EventLog;
use rand::{rngs::StdRng, SeedableRng};

use crate::{
    mutation::{LogMutator, MutationResult},
    parsing::traits::DirName,
    utils::sampling::{sample_log_with_replacement_mut, sample_log_without_replacement_mut},
};

/// Mutator to create a new log by randomly sampling cases with replacement.
/// The sampled cases are assigned unique case ids ("0" ... "`size`").
#[derive(DirName)]
pub struct LogBootstrapper {
    /// The number of cases to sample.
    #[dirname(rename = "")]
    size: usize,
    /// Sample with replacement? Defaults to true.
    replacement: bool,
    /// Optional seed for the random case sampling. Ensures reproducible results
    /// across runs. Use [`LogBootstrapper::with_seed`] to set the seed.
    seed: Option<u64>,
    #[dirname(ignore)]
    rng: StdRng,
}

impl LogBootstrapper {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            replacement: true,
            seed: None,
            rng: StdRng::from_entropy(),
        }
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self.rng = StdRng::seed_from_u64(seed);
        self
    }
}

impl LogMutator for LogBootstrapper {
    fn apply_mut(&mut self, log: &mut EventLog) -> MutationResult<()> {
        if self.replacement {
            sample_log_with_replacement_mut(&mut self.rng, log, self.size);
        } else {
            sample_log_without_replacement_mut(&mut self.rng, log, self.size);
        }
        Ok(())
    }
}

impl LogBootstrapper {
    pub fn with_replacement(mut self, replacement: bool) -> Self {
        self.replacement = replacement;
        self
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use process_mining::{
        event_log::{Attribute, AttributeValue},
        EventLog,
    };

    use crate::{
        mutation::LogMutator,
        test_fixtures::abcd_log,
        utils::attributes::{get_string_by_key, get_traceid, get_traceids},
    };

    use super::LogBootstrapper;
    use rstest::rstest;

    #[rstest]
    #[should_panic]
    fn sample_without_replacement_fails_with_large_size(abcd_log: EventLog) {
        LogBootstrapper::new(5)
            .with_replacement(false)
            .apply(&abcd_log)
            .unwrap();
    }

    #[rstest]
    fn sample_without_replacement_has_no_duplicates(abcd_log: EventLog) {
        let trace_ids = get_traceids(&abcd_log).unwrap();
        //
        // Do it a couple of times to make (more) sure that we aren't getting lucky
        for _ in 1..10 {
            let mutated_log = LogBootstrapper::new(4)
                .with_replacement(false)
                .apply(&abcd_log)
                .unwrap();
            let new_traceids = get_traceids(&mutated_log).unwrap();

            assert_eq!(trace_ids, new_traceids);
        }
    }

    #[rstest]
    fn unseeded_sample_without_replacement_is_random(abcd_log: EventLog) {
        let mut seen_trace_ids: HashSet<String> = HashSet::new();

        // Test that sampling multiple times yields different results.
        for _ in 1..10 {
            let mutated_log = LogBootstrapper::new(1)
                .with_replacement(false)
                .apply(&abcd_log)
                .unwrap();
            seen_trace_ids = seen_trace_ids
                .union(&get_traceids(&mutated_log).unwrap())
                .cloned()
                .collect();
        }

        assert!(seen_trace_ids.len() > 1);
    }

    #[rstest]
    fn sample_with_replacement_has_duplicates(mut abcd_log: EventLog) {
        abcd_log.traces.iter_mut().for_each(|trace| {
            let traceid = get_traceid(trace).unwrap();
            trace.attributes.push(Attribute {
                key: "original_traceid".to_string(),
                value: AttributeValue::String(traceid),
                own_attributes: None,
            });
        });

        // Don't explicitly specify the
        let mutated_log = LogBootstrapper::new(1000)
            .with_replacement(true)
            .apply(&abcd_log)
            .unwrap();

        let mut traceids: Vec<String> = mutated_log
            .traces
            .iter()
            .map(|trace| get_string_by_key(trace, "original_traceid").unwrap())
            .collect();
        traceids.sort();

        let has_dups = traceids.windows(2).any(|window| window[0] == window[1]);

        assert!(has_dups);
    }

    #[test]
    fn default_is_with_replacement() {
        let mutator = LogBootstrapper::new(10);
        assert!(mutator.replacement);
    }

    #[rstest]
    fn seeded_gives_same_result(mut abcd_log: EventLog) {
        abcd_log.traces.iter_mut().for_each(|trace| {
            let traceid = get_traceid(trace).unwrap();
            trace.attributes.push(Attribute {
                key: "original_traceid".to_string(),
                value: AttributeValue::String(traceid),
                own_attributes: None,
            });
        });

        let new_log_1 = LogBootstrapper::new(1000)
            .with_replacement(true)
            .with_seed(42)
            .apply(&abcd_log)
            .unwrap();
        let new_log_2 = LogBootstrapper::new(1000)
            .with_replacement(true)
            .with_seed(42)
            .apply(&abcd_log)
            .unwrap();

        let log_1_caseids: Vec<_> = new_log_1
            .traces
            .iter()
            .map(|trace| get_string_by_key(trace, "original_traceid").unwrap())
            .collect();

        let log_2_caseids: Vec<_> = new_log_2
            .traces
            .iter()
            .map(|trace| get_string_by_key(trace, "original_traceid").unwrap())
            .collect();

        assert_eq!(log_1_caseids, log_2_caseids);
    }
}
