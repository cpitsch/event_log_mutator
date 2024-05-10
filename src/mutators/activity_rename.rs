use process_mining::event_log::{AttributeValue, Event, Trace};
use rand::random;

use crate::{
    constants::NO_ACTIVITY_LABEL_MSG,
    mutation::TraceMutator,
    utils::{get_activity_label, set_activity_label},
};

pub struct ActivityRenamer {
    /// The activity to rename. This modifier will only effect events with this label.
    activity: String,
    /// The new activity label.
    new_label: String,
    /// The probability of renaming. Ranges from 0 to 1.
    probability: f32,
}

impl ActivityRenamer {
    pub fn new(activity: impl Into<String>, new_label: impl Into<String>) -> Self {
        Self {
            activity: activity.into(),
            new_label: new_label.into(),
            probability: 1.0,
        }
    }

    fn should_mutate(&self, event: &Event) -> bool {
        get_activity_label(event).expect(NO_ACTIVITY_LABEL_MSG) == self.activity
            && random::<f32>() < self.probability
    }

    pub fn with_probability(mut self, probability: f32) -> Self {
        self.probability = probability;
        self
    }
}

impl TraceMutator for ActivityRenamer {
    fn apply(&self, trace: &Trace) -> Trace {
        let mut new_trace = trace.clone();
        new_trace.events.iter_mut().for_each(|evt| {
            if self.should_mutate(evt) {
                set_activity_label(evt, AttributeValue::String(self.new_label.clone()));
            }
        });
        new_trace
    }
}
