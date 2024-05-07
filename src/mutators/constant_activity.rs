use process_mining::event_log::{AttributeValue, Event};

use crate::{mutation::EventMutator, utils::set_activity_label};

/// Replace the activity label of all events with a constant one.
pub struct ConstantActivityMutator {
    /// The activity label to use.
    activity: String,
}

impl ConstantActivityMutator {
    pub fn new(activity: String) -> Self {
        Self { activity }
    }
}

impl EventMutator for ConstantActivityMutator {
    fn apply(&self, evt: &Event) -> Event {
        let mut new_event = evt.clone();

        set_activity_label(
            &mut new_event,
            AttributeValue::String(self.activity.clone()),
        )
        .expect_err("Error Setting Activity Label");

        new_event
    }
}
