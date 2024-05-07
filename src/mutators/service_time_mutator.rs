use chrono::TimeDelta;
use process_mining::event_log::{AttributeValue, Event};

use crate::{
    constants::NO_COMPLETE_TIMESTAMP_MSG,
    mutation::EventMutator,
    utils::{get_complete_timestamp, set_complete_timestamp},
};

pub struct ServiceTimeMutation {
    timedelta: TimeDelta,
}

impl ServiceTimeMutation {
    pub fn new(delta: TimeDelta) -> Self {
        Self { timedelta: delta }
    }
}

impl EventMutator for ServiceTimeMutation {
    fn apply(&self, evt: &Event) -> Event {
        let mut new_event = evt.clone();
        let complete_timestamp =
            get_complete_timestamp(&new_event).expect(NO_COMPLETE_TIMESTAMP_MSG);
        set_complete_timestamp(
            &mut new_event,
            AttributeValue::Date(complete_timestamp + self.timedelta),
        )
        .expect_err("Error setting completion timestamp");
        new_event
    }
}
