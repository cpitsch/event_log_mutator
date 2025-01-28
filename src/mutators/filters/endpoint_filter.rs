use process_mining::{event_log::Trace, EventLog};

use crate::{
    mutation::{LogMutator, MutationError, MutationResult},
    mutators::DisplayVec,
    parsing::traits::DirName,
    utils::{
        attributes::{get_activities, get_end_activities, get_start_activities, AttributeResult},
        errors::retain_err,
    },
};

/// Mutation to retain only the cases which start or end with certain activities
#[derive(DirName)]
pub struct EndpointFilter {
    /// The starting activities to filter for. Defaults to all activities (no cases
    /// filtered).
    start_activities: Option<DisplayVec<String>>,
    /// The end activities to filter for. Defaults to all activities (no cases
    /// filtered).
    end_activities: Option<DisplayVec<String>>,
}

impl EndpointFilter {
    pub fn new(
        start_activities: Option<impl Into<DisplayVec<String>>>,
        end_activities: Option<impl Into<DisplayVec<String>>>,
    ) -> Self {
        EndpointFilter {
            start_activities: start_activities.map(|acts| acts.into()),
            end_activities: end_activities.map(|acts| acts.into()),
        }
    }

    fn keep_trace(
        &self,
        trace: &Trace,
        start_activities: &[String],
        end_activities: &[String],
    ) -> AttributeResult<bool> {
        // Searches for all activities with the start timestamp so that even if an
        // event occurs second, but at the same time as the first event, it counts
        // for the filter
        let trace_start_acts = get_start_activities(trace)?;
        let trace_end_acts = get_end_activities(trace)?;

        Ok(start_activities
            .iter()
            .any(|item| trace_start_acts.contains(item))
            && end_activities
                .iter()
                .any(|item| trace_end_acts.contains(item)))
    }
}

impl LogMutator for EndpointFilter {
    fn apply_mut(&mut self, log: &mut EventLog) -> MutationResult<()> {
        let all_activities: Vec<String> = get_activities(log)
            .map_err(|e| MutationError::MissingAttributeError("EndpointFilter", e))?
            .into_iter()
            .collect();

        let start_acts = self
            .start_activities
            .as_deref()
            .unwrap_or(all_activities.as_slice());
        let end_acts = self
            .end_activities
            .as_deref()
            .unwrap_or(all_activities.as_slice());

        retain_err(&mut log.traces, |trace| {
            self.keep_trace(trace, start_acts, end_acts)
        })
        .map_err(|e| MutationError::MissingAttributeError("EndpointFilter", e))?;
        Ok(())
    }
}
