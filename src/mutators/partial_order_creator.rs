use process_mining::event_log::{AttributeValue, Trace};

use crate::{
    constants::NO_COMPLETE_TIMESTAMP_MSG,
    mutation::TraceMutator,
    utils::{get_complete_timestamp, set_start_timestamp},
};

/// Mutation to add service time information to an event log by assuming the timespan
/// between two events completing to be the service time of the second event
#[derive(Default)]
pub struct PartialOrderCreator;

impl PartialOrderCreator {
    pub fn new() -> Self {
        Self {}
    }
}

impl TraceMutator for PartialOrderCreator {
    fn apply(&self, trace: &Trace) -> Trace {
        let mut new_trace = trace.clone();

        if let Some(evt) = new_trace.events.get_mut(0) {
            // Set its start_timestamp to its completion timestamp since we have no
            // information on this. This means the first event always has service time 0.
            set_start_timestamp(
                evt,
                AttributeValue::Date(get_complete_timestamp(evt).expect(NO_COMPLETE_TIMESTAMP_MSG)),
            );
        }

        // Use trace instead of new_trace, because we do not change the complete
        // timestamps anyways, and this allows for this nice zip and iter_mut combintation
        trace
            .events
            .iter()
            .zip(new_trace.events.iter_mut().skip(1))
            .for_each(|(e1, e2)| {
                let e1_complete_timestamp =
                    get_complete_timestamp(e1).expect(NO_COMPLETE_TIMESTAMP_MSG);
                set_start_timestamp(e2, AttributeValue::Date(e1_complete_timestamp));
            });
        new_trace
    }
}
