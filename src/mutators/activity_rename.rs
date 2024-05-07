use process_mining::event_log::{AttributeValue, Trace};
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
    pub fn new(activity: String, new_label: String, probability: f32) -> Self {
        Self {
            activity,
            new_label,
            probability,
        }
    }
}

impl TraceMutator for ActivityRenamer {
    fn apply(&self, trace: &Trace) -> Trace {
        let mut new_trace = trace.clone();
        new_trace.events.iter_mut().for_each(|evt| {
            if get_activity_label(evt).expect(NO_ACTIVITY_LABEL_MSG) == self.activity
                && random::<f32>() < self.probability
            {
                set_activity_label(evt, AttributeValue::String(self.new_label.clone())).unwrap();
            }
        });
        new_trace
    }
}
