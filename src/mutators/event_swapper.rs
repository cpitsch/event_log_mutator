use itertools::Itertools;
use process_mining::event_log::{AttributeValue, Trace};
use rand::random;

use crate::{
    constants::NO_START_TIMESTAMP_MSG,
    mutation::TraceMutator,
    parsing::as_dir_name::AsDirName,
    utils::{
        get_activity_label, get_service_time, get_start_timestamp, set_complete_timestamp,
        set_start_timestamp,
    },
};

/// Swap pairs of events with activity label `activity_1` and `activity_2`.
/// First occurrence of `activity_1` is swapped with first occurrence of `activity_2`,
/// etc.
/// Un-paired events are not affected.
#[derive(AsDirName)]
pub struct EventSwapper {
    // The first activity label for swapping.
    #[asdirname(rename = "")]
    activity_1: String,
    // The second activity label for swapping.
    #[asdirname(rename = "swap")]
    activity_2: String,
    /// The probability of applying this modifier (per pair)
    #[asdirname(rename = "p", no_split)]
    probability: f32,
}

impl EventSwapper {
    pub fn new(activity_1: impl Into<String>, activity_2: impl Into<String>) -> EventSwapper {
        Self {
            activity_1: activity_1.into(),
            activity_2: activity_2.into(),
            probability: 1.0,
        }
    }

    // fn should_mutate(&self, trace: &Trace, idx_1: &usize, idx_2: &usize) -> bool {
    fn should_mutate(&self) -> bool {
        random::<f32>() < self.probability
    }

    pub fn with_probability(mut self, probability: f32) -> Self {
        self.probability = probability;
        self
    }
}

impl TraceMutator for EventSwapper {
    fn apply(&self, trace: &Trace) -> Trace {
        let mut new_trace = trace.clone();

        // Get all indices of activity 1 and 2
        let act_1_indices = new_trace
            .events
            .iter()
            .positions(|evt| get_activity_label(evt) == Some(self.activity_1.clone()))
            .collect_vec();
        let act_2_indices = new_trace
            .events
            .iter()
            .positions(|evt| get_activity_label(evt) == Some(self.activity_2.clone()))
            .collect_vec();

        act_1_indices
            .iter()
            .zip(act_2_indices.iter())
            .for_each(|(idx_1, idx_2)| {
                if self.should_mutate() {
                    // Swap their start_timestamp, and update their complete timestamp
                    // based on their service time
                    let event_1_start = get_start_timestamp(new_trace.events.get(*idx_1).unwrap())
                        .expect(NO_START_TIMESTAMP_MSG);
                    let event_2_start = get_start_timestamp(new_trace.events.get(*idx_2).unwrap())
                        .expect(NO_START_TIMESTAMP_MSG);

                    let evt_1_service_time =
                        get_service_time(new_trace.events.get(*idx_1).unwrap()).unwrap();
                    let evt_2_service_time =
                        get_service_time(new_trace.events.get(*idx_2).unwrap()).unwrap();

                    // Swap start timestamps
                    set_start_timestamp(
                        new_trace.events.get_mut(*idx_1).unwrap(),
                        AttributeValue::Date(event_2_start),
                    );
                    set_start_timestamp(
                        new_trace.events.get_mut(*idx_2).unwrap(),
                        AttributeValue::Date(event_1_start),
                    );
                    // Update complete timestamps to match old service time
                    set_complete_timestamp(
                        new_trace.events.get_mut(*idx_1).unwrap(),
                        AttributeValue::Date(event_2_start + evt_1_service_time),
                    );
                    set_complete_timestamp(
                        new_trace.events.get_mut(*idx_2).unwrap(),
                        AttributeValue::Date(event_1_start + evt_2_service_time),
                    );

                    // Swap them in the trace events vec
                    new_trace.events.swap(*idx_1, *idx_2);
                }
            });

        new_trace
    }
}
