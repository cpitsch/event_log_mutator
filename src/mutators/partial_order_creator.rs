use process_mining::event_log::{AttributeValue, Trace};

use crate::{
    constants::NO_COMPLETE_TIMESTAMP_MSG,
    mutation::TraceMutator,
    parsing::dir_name_trait::DirName,
    utils::attributes::{get_complete_timestamp, set_start_timestamp},
};

/// Mutation to add service time information to an event log by assuming the timespan
/// between two events completing to be the service time of the second event
#[derive(Default, DirName)]
pub struct PartialOrderCreator;

impl PartialOrderCreator {
    pub fn new() -> Self {
        Self {}
    }
}

impl TraceMutator for PartialOrderCreator {
    fn apply_mut(&mut self, trace: &mut Trace) {
        if let Some(evt) = trace.events.get_mut(0) {
            // Set its start_timestamp to its completion timestamp since we have no
            // information on this. This means the first event always has service time 0.
            let first_complete_timestamp =
                get_complete_timestamp(evt).expect(NO_COMPLETE_TIMESTAMP_MSG);
            set_start_timestamp(evt, AttributeValue::Date(first_complete_timestamp));

            let mut previous_timestamp = first_complete_timestamp;

            for event in trace.events.iter_mut().skip(1) {
                set_start_timestamp(event, AttributeValue::Date(previous_timestamp));
                previous_timestamp =
                    get_complete_timestamp(event).expect(NO_COMPLETE_TIMESTAMP_MSG);
            }
        }
    }
}
