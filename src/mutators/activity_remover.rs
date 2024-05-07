use itertools::Itertools;
use process_mining::event_log::Trace;
use rand::random;

use crate::{constants::NO_ACTIVITY_LABEL_MSG, mutation::TraceMutator, utils::get_activity_label};

/// Mutator to remove events that have the given activity label.
pub struct ActivityRemover {
    /// The activity label to remove.
    activity: String,
    /// The probability of removal. Ranges from 0 to 1.
    probability: f32,
}

impl ActivityRemover {
    pub fn new(activity: String, probability: f32) -> Self {
        Self {
            activity,
            probability,
        }
    }
}

impl TraceMutator for ActivityRemover {
    fn apply(&self, trace: &Trace) -> Trace {
        let mut new_trace = trace.clone();
        new_trace.events = new_trace
            .events
            .iter()
            .filter(|evt| {
                if get_activity_label(evt).expect(NO_ACTIVITY_LABEL_MSG) == self.activity {
                    // Matching activity - skip this event if rnd < prob
                    random::<f32>() >= self.probability
                } else {
                    true
                }
            })
            .cloned()
            .collect_vec();

        new_trace
    }
}
