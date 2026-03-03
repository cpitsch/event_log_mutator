use crate::{
    mutation::{EventMutator, MutationResult},
    parsing::traits::DirName,
};
use process_mining::core::event_data::case_centric::Event;

/// A mutation to remove an attribute key from all events.
#[derive(DirName)]
pub struct AttributeRemover {
    /// The key to remove
    #[dirname(rename = "")]
    key: String,
}

impl AttributeRemover {
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into() }
    }
}

impl EventMutator for AttributeRemover {
    fn apply_mut(&mut self, evt: &mut Event) -> MutationResult<()> {
        evt.attributes.retain(|attr| attr.key != self.key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::AttributeRemover;
    use crate::mutation::TraceMutator;
    use crate::test_fixtures::abcd_trace;
    use process_mining::core::event_data::case_centric::Trace;
    use rstest::rstest;

    #[rstest]
    #[case::remove_concept_name("concept:name")]
    #[case::remove_start_timestamp("start_timestamp")]
    #[case::remove_complete_timestamp("time:timestamp")]
    fn correct_key_is_always_removed(abcd_trace: Trace, #[case] key: String) {
        let mut remaining_keys = HashSet::from([
            "concept:name".to_string(),
            "time:timestamp".to_string(),
            "start_timestamp".to_string(),
        ]);
        remaining_keys.retain(|k| k != &key);

        let new_trace = AttributeRemover::new(key).apply(&abcd_trace).unwrap();

        // All events have the appropriate key removed
        new_trace.events.iter().for_each(|evt| {
            let attributes: HashSet<String> =
                evt.attributes.iter().map(|attr| attr.key.clone()).collect();
            assert_eq!(attributes, remaining_keys);
        });
    }

    #[rstest]
    fn nonexistent_attribute_doesnt_panic(abcd_trace: Trace) {
        // This should not panic
        let _ = AttributeRemover::new("DOESNT_EXIST").apply(&abcd_trace);
    }
}
