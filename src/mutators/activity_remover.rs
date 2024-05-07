use itertools::Itertools;
use process_mining::event_log::{Event, Trace};
use rand::random;

use crate::{constants::NO_ACTIVITY_LABEL_MSG, mutation::TraceMutator, utils::get_activity_label};

/// Mutator to remove events that have the given activity label.
pub struct ActivityRemover {
    /// The activity label to remove.
    activity: String,
    /// The probability of removal. Ranges from 0 to 1. Defaults to 1
    probability: f32,
}

impl ActivityRemover {
    pub fn new(activity: String) -> Self {
        Self {
            activity,
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
        new_trace.events = new_trace
            .events
            .iter()
            .filter(|evt| !self.should_remove(evt))
            .cloned()
            .collect_vec();

        new_trace
    }
}
