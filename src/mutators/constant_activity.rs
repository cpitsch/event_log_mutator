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
