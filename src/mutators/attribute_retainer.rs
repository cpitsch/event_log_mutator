use itertools::Itertools;
use process_mining::event_log::Event;

use crate::mutation::{EventMutator, MutationResult};
use crate::parsing::traits::DirName;

use super::DisplayVec;

/// A Mutation to remove all attributes from events, except a specified set
#[derive(DirName)]
pub struct AttributeRetainer {
    /// The keys to keep
    #[dirname(ignore)]
    attributes: DisplayVec<String>,
}

impl AttributeRetainer {
    pub fn new(attributes: impl Into<DisplayVec<String>>) -> Self {
        Self {
            attributes: attributes.into(),
        }
    }
}

impl EventMutator for AttributeRetainer {
    fn apply_mut(&mut self, evt: &mut Event) -> MutationResult<()> {
        evt.attributes
            .retain(|attr| self.attributes.0.iter().contains(&attr.key));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::AttributeRetainer;
    use crate::mutation::EventMutator;
    use crate::test_fixtures::new_event;
    use chrono::{TimeDelta, Utc};
    use rstest::rstest;

    #[rstest]
    #[case::retain_concept_name(vec!["concept:name".into()])]
    #[case::retain_time_and_activity(vec!["concept:name".into(), "time:timestamp".into()])]
    #[case::retain_none(vec![])]
    // In this case, actually no attributes will be left
    #[case::nonexistent(vec!["I don't exist".into()])]
    fn correct_keys_retained(#[case] keys: Vec<String>) {
        let the_event = new_event("a", Utc::now().fixed_offset(), TimeDelta::hours(1));
        // Sanity check to ensure new_event works as expected
        let before_attrs = HashSet::from([
            "concept:name".to_string(),
            "time:timestamp".to_string(),
            "start_timestamp".to_string(),
        ]);
        assert_eq!(
            the_event
                .attributes
                .iter()
                .map(|attr| attr.key.clone())
                .collect::<HashSet<_>>(),
            before_attrs
        );

        let new_event = AttributeRetainer::new(keys.clone())
            .apply(&the_event)
            .unwrap();

        let expected: HashSet<_> = keys
            .into_iter()
            .collect::<HashSet<_>>()
            .intersection(&before_attrs)
            .cloned()
            .collect();

        assert_eq!(
            new_event
                .attributes
                .into_iter()
                .map(|attr| attr.key)
                .collect::<HashSet<_>>(),
            expected
        );
    }
}
