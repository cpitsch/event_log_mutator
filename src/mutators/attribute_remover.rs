use crate::{mutation::EventMutator, parsing::dir_name_trait::DirName};

/// A Mutation to remove an attribute key from all events.
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
    fn apply(&self, evt: &process_mining::event_log::Event) -> process_mining::event_log::Event {
        let mut new_event = evt.clone();
        new_event.attributes.retain(|attr| attr.key != self.key);
        new_event
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::mutation::TraceMutator;
    use crate::test_fixtures::abcd_trace;
    use process_mining::event_log::Trace;
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

        let mutator = AttributeRemover::new(key);
        let new_trace = TraceMutator::apply(&mutator, &abcd_trace);

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
        let _ = TraceMutator::apply(&AttributeRemover::new("DOESNT_EXIST"), &abcd_trace);
    }
}
