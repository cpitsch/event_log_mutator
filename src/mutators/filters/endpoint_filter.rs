use std::fmt::Display;

use itertools::Itertools;
use process_mining::event_log::Trace;

use crate::{
    mutation::LogMutator,
    parsing::dir_name_trait::DirName,
    utils::{get_activities, get_end_activities, get_start_activities},
};

#[derive(DirName)]
pub struct EndpointFilter {
    start_activities: Option<DisplayVec<String>>,
    end_activities: Option<DisplayVec<String>>,
}

#[derive(Clone)]
struct DisplayVec<T: Display>(Vec<T>);

impl<T: Display> Display for DisplayVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().join("+"))
    }
}

impl EndpointFilter {
    pub fn new(start_activities: Option<Vec<String>>, end_activities: Option<Vec<String>>) -> Self {
        EndpointFilter {
            start_activities: start_activities.map(DisplayVec),
            end_activities: end_activities.map(DisplayVec),
        }
    }

    fn keep_trace(
        &self,
        trace: &Trace,
        start_activities: &[String],
        end_activities: &[String],
    ) -> bool {
        // Searches for all activities with the start timestamp so that even if an
        // event occurs second, but at the same time as the first event, it counts
        // for the filter
        let trace_start_acts = get_start_activities(trace);
        let trace_end_acts = get_end_activities(trace);

        start_activities
            .iter()
            .any(|item| trace_start_acts.contains(item))
            && end_activities
                .iter()
                .any(|item| trace_end_acts.contains(item))
    }
}

impl LogMutator for EndpointFilter {
    fn apply(&self, log: &process_mining::EventLog) -> process_mining::EventLog {
        let mut new_log = log.clone();
        let all_activities: Vec<String> = get_activities(log).into_iter().collect();

        let start_acts = self
            .start_activities
            .clone()
            .map(|v| v.0)
            .unwrap_or(all_activities.clone());
        let end_acts = self
            .end_activities
            .clone()
            .map(|v| v.0)
            .unwrap_or(all_activities);

        new_log
            .traces
            .retain(|trace| self.keep_trace(trace, &start_acts, &end_acts));

        new_log
    }
}
