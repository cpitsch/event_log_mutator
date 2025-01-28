use process_mining::{event_log::Trace, EventLog};

use crate::{
    mutation::{LogMutator, MutationError, MutationResult},
    mutators::DisplayVec,
    parsing::traits::DirName,
    utils::{
        attributes::{get_activities, get_activity_label, AttributeResult},
        errors::retain_err,
    },
};

/// Mutation to retain only the cases which contain eventually-follows relations between certain activities
#[derive(DirName)]
pub struct FollowerFilter {
    /// The relation "trigger" activities to filter for. Defaults to all activities (no cases
    /// filtered).
    trigger_activities: Option<DisplayVec<String>>,
    /// The relation "reaction" activities to filter for. Defaults to all activities (no cases
    /// filtered).
    reaction_activities: Option<DisplayVec<String>>,
    /// The window (number of events) to accept for an eventually-follows relation.
    /// For instance, a range of 1 means we only consider Directly-Follows relations.
    /// Defaults to the trace length
    range: Option<usize>,
    // TODO: Add a parameter to restrict the time delay between the two events?
}

impl FollowerFilter {
    pub fn new(
        trigger_activities: Option<impl Into<DisplayVec<String>>>,
        reaction_activities: Option<impl Into<DisplayVec<String>>>,
    ) -> Self {
        FollowerFilter {
            trigger_activities: trigger_activities.map(|acts| acts.into()),
            reaction_activities: reaction_activities.map(|acts| acts.into()),
            range: None,
        }
    }

    pub fn with_range(mut self, range: usize) -> Self {
        self.range = Some(range);
        self
    }

    fn keep_trace(
        &self,
        trace: &Trace,
        trigger_activities: &[String],
        reaction_activities: &[String],
    ) -> AttributeResult<bool> {
        let range = self.range.unwrap_or(trace.events.len());
        let trace_activities: Vec<String> = trace
            .events
            .iter()
            .map(get_activity_label)
            .collect::<AttributeResult<Vec<String>>>()?;

        let is_trigger_act: Vec<bool> = trace_activities
            .iter()
            .map(|act| trigger_activities.contains(act))
            .collect();
        let is_reaction_act: Vec<bool> = trace_activities
            .iter()
            .map(|act| reaction_activities.contains(act))
            .collect();

        let result = (0..trace_activities.len() - 1).any(|i| {
            is_trigger_act[i]
                && is_reaction_act[i + 1..(i + range + 1).min(is_reaction_act.len())]
                    .iter()
                    .any(|x| *x)
        });
        Ok(result)
    }
}

impl LogMutator for FollowerFilter {
    fn apply_mut(&mut self, log: &mut EventLog) -> MutationResult<()> {
        let all_activities: Vec<String> = get_activities(log)
            .map_err(|e| MutationError::MissingAttributeError("EndpointFilter", e))?
            .into_iter()
            .collect();

        let trigger_acts = self
            .trigger_activities
            .as_deref()
            .unwrap_or(all_activities.as_slice());
        let react_acts = self
            .reaction_activities
            .as_deref()
            .unwrap_or(all_activities.as_slice());

        retain_err(&mut log.traces, |trace| {
            self.keep_trace(trace, trigger_acts, react_acts)
        })
        .map_err(|e| MutationError::MissingAttributeError("EndpointFilter", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::abcd_trace;
    use process_mining::event_log::Trace;
    use rstest::rstest;

    #[rstest]
    fn test_keep_trace(abcd_trace: Trace) {
        let filter_1 =
            FollowerFilter::new(Some(vec!["a".to_string()]), Some(vec!["c".to_string()]))
                .with_range(1);

        // It is not directly (1) following, but eventually (2)
        assert!(!filter_1
            .keep_trace(&abcd_trace, &["a".to_string()], &["c".to_string()])
            .unwrap());

        let filter_2 =
            FollowerFilter::new(Some(vec!["a".to_string()]), Some(vec!["c".to_string()]))
                .with_range(2);

        // This should succeed since it is exactly a distance of 2
        assert!(filter_2
            .keep_trace(&abcd_trace, &["a".to_string()], &["c".to_string()])
            .unwrap());

        let filter_3 =
            FollowerFilter::new(Some(vec!["a".to_string()]), Some(vec!["c".to_string()]))
                .with_range(3);

        // Distance of 2 should still count for larger windows
        assert!(filter_3
            .keep_trace(&abcd_trace, &["a".to_string()], &["c".to_string()])
            .unwrap());
    }

    #[rstest]
    fn test_no_matches_discards(abcd_trace: Trace) {
        let filter = FollowerFilter::new(Some(vec!["y".to_string()]), Some(vec!["z".to_string()]))
            .with_range(4); // 4 = Entire trace

        // Should discard the trace since this relation does not exist in the trace
        // since the activities "y" and "z" do not exist in the trace.
        assert!(!filter
            .keep_trace(&abcd_trace, &["y".to_string()], &["z".to_string()])
            .unwrap());
    }
}
