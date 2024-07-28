use process_mining::event_log::{AttributeValue, Event};
use rand::random;

use crate::{mutation::EventMutator, parsing::dir_name_trait::DirName, utils::set_activity_label};

/// Replace the activity label of all events with a constant one.
#[derive(DirName)]
pub struct ConstantActivityMutator {
    /// The activity label to use.
    #[dirname(rename = "")]
    activity: String,
    /// The probability of applying the mutation to an event
    #[dirname(rename = "p", no_split)]
    probability: f32,
}

impl ConstantActivityMutator {
    pub fn new(activity: impl Into<String>) -> Self {
        Self {
            activity: activity.into(),
            probability: 1.0,
        }
    }

    fn should_mutate(&self) -> bool {
        random::<f32>() < self.probability
    }

    pub fn with_probability(mut self, probability: f32) -> Self {
        self.probability = probability;
        self
    }
}

impl EventMutator for ConstantActivityMutator {
    fn apply(&self, evt: &Event) -> Event {
        if self.should_mutate() {
            let mut new_event = evt.clone();

            set_activity_label(
                &mut new_event,
                AttributeValue::String(self.activity.clone()),
            );

            new_event
        } else {
            evt.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{mutation::TraceMutator, test_fixtures::abcd_trace, utils::get_activity_label};
    use process_mining::event_log::Trace;
    use rstest::rstest;

    #[rstest]
    fn all_events_rename(abcd_trace: Trace) {
        let mutator = ConstantActivityMutator::new("NEW_ACTIVITY".to_string());
        let new_trace = TraceMutator::apply(&mutator, &abcd_trace);

        assert!(new_trace
            .events
            .iter()
            .all(|evt| get_activity_label(evt).unwrap() == *"NEW_ACTIVITY"));
    }
}
