use process_mining::event_log::{Event, Trace};
use rand::random;

use crate::{
    constants::NO_ACTIVITY_LABEL_MSG, mutation::TraceMutator, parsing::dir_name_trait::DirName,
    utils::get_activity_label,
};

/// Mutator to remove events that have the given activity label.
#[derive(DirName)]
pub struct ActivityRemover {
    /// The activity label to remove.
    #[dirname(rename = "")]
    activity: String,
    /// The probability of removal. Ranges from 0 to 1. Defaults to 1

    #[dirname(rename = "p", no_split)]
    probability: f32,
}

impl ActivityRemover {
    pub fn new(activity: impl Into<String>) -> Self {
        Self {
            activity: activity.into(),
            probability: 1.0,
        }
    }

    fn should_remove(&self, event: &Event) -> bool {
        get_activity_label(event).expect(NO_ACTIVITY_LABEL_MSG) == self.activity
            && random::<f32>() < self.probability
    }

    pub fn with_probability(mut self, probability: f32) -> Self {
        self.probability = probability;
        self
    }
}

impl TraceMutator for ActivityRemover {
    fn apply(&self, trace: &Trace) -> Trace {
        let mut new_trace = trace.clone();
        new_trace.events.retain(|evt| !self.should_remove(evt));
        new_trace
    }
}
