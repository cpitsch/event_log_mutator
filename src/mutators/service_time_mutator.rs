use chrono::TimeDelta;
use process_mining::event_log::{AttributeValue, Event};
use rand::random;

use crate::{
    constants::{NO_ACTIVITY_LABEL_MSG, NO_COMPLETE_TIMESTAMP_MSG},
    mutation::EventMutator,
    utils::{get_activity_label, get_complete_timestamp, set_complete_timestamp},
};

/// Mutation to increase the service time by a constant amount.
pub struct ServiceTimeMutation {
    /// Only apply the mutation to events with this activity. Defaults to all activities.
    /// Use [`ServiceTimeMutation::for_activity`] to set a specific activity.
    activity: Option<String>,
    /// The probability to apply the mutation to a matching event. Ranges from 0 to 1.
    /// Use [`ServiceTimeMutation::with_probability`] to set a probability.
    probability: f32,
    /// The time difference to add to the service time.
    timedelta: TimeDelta,
}

impl ServiceTimeMutation {
    pub fn new(delta: TimeDelta) -> Self {
        Self {
            activity: None,
            probability: 1.0,
            timedelta: delta,
        }
    }

    fn should_mutate(&self, event: &Event) -> bool {
        (
            // Check that the event matches the requirements
            self.activity.clone().map_or(true, |act| {
                get_activity_label(event).expect(NO_ACTIVITY_LABEL_MSG) == act
            })
        ) && (
            // Check mutation probability
            random::<f32>() < self.probability
        )
    }

    pub fn for_activity(mut self, activity: String) -> Self {
        self.activity = Some(activity);
        self
    }

    pub fn with_probability(mut self, probability: f32) -> Self {
        self.probability = probability;
        self
    }
}

impl EventMutator for ServiceTimeMutation {
    fn apply(&self, evt: &Event) -> Event {
        if self.should_mutate(evt) {
            let mut new_event = evt.clone();
            let complete_timestamp =
                get_complete_timestamp(&new_event).expect(NO_COMPLETE_TIMESTAMP_MSG);
            set_complete_timestamp(
                &mut new_event,
                AttributeValue::Date(complete_timestamp + self.timedelta),
            )
            .expect_err("Error setting completion timestamp");
            new_event
        } else {
            evt.clone()
        }
    }
}
