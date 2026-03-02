use process_mining::core::event_data::case_centric::{EventLog, Trace};

use crate::{
    mutation::{LogMutator, MutationError, MutationResult},
    mutators::DisplayVec,
    parsing::traits::DirName,
    utils::{
        attributes::{get_activity_label, AttributeResult},
        errors::retain_err,
    },
};

/// Mutation to retain only the cases which contain eventually-follows relations between certain activities
#[derive(DirName)]
pub struct FollowerFilter {
    /// The relation "trigger" activities to filter for.
    trigger_activities: DisplayVec<String>,
    /// The relation "reaction" activities to filter for.
    reaction_activities: DisplayVec<String>,
    /// The window (number of events) to accept for an eventually-follows relation.
    /// For instance, a range of 1 means we only consider Directly-Follows relations.
    /// Defaults to the trace length
    range: Option<usize>,
    // TODO: Add a parameter to restrict the time delay between the two events?
}

impl FollowerFilter {
    pub fn new(
        trigger_activities: impl Into<DisplayVec<String>>,
        reaction_activities: impl Into<DisplayVec<String>>,
    ) -> Self {
        Self {
            trigger_activities: trigger_activities.into(),
            reaction_activities: reaction_activities.into(),
            range: None,
        }
    }

    pub fn with_range(mut self, range: usize) -> Self {
        self.range = Some(range);
        self
    }

    fn keep_trace(&self, trace: &Trace) -> AttributeResult<bool> {
        let range = self.range.unwrap_or(trace.events.len());
        let trace_activities: Vec<&String> = trace
            .events
            .iter()
            .map(get_activity_label)
            .collect::<AttributeResult<_>>()?;

        let is_trigger_act: Vec<bool> = trace_activities
            .iter()
            .map(|act| self.trigger_activities.contains(act))
            .collect();
        let is_reaction_act: Vec<bool> = trace_activities
            .iter()
            .map(|act| self.reaction_activities.contains(act))
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
        retain_err(&mut log.traces, |trace| self.keep_trace(trace))
            .map_err(|e| MutationError::AttributeError("EndpointFilter", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::abcd_trace;
    use process_mining::{core::event_data::case_centric::Trace, event_log, trace};
    use rstest::rstest;

    #[rstest]
    fn test_keep_trace(abcd_trace: Trace) {
        let filter_1 =
            FollowerFilter::new(vec!["a".to_string()], vec!["c".to_string()]).with_range(1);

        // It is not directly (1) following, but eventually (2)
        assert!(!filter_1.keep_trace(&abcd_trace).unwrap());

        let filter_2 =
            FollowerFilter::new(vec!["a".to_string()], vec!["c".to_string()]).with_range(2);

        // This should succeed since it is exactly a distance of 2
        assert!(filter_2.keep_trace(&abcd_trace).unwrap());

        let filter_3 =
            FollowerFilter::new(vec!["a".to_string()], vec!["c".to_string()]).with_range(3);

        // Distance of 2 should still count for larger windows
        assert!(filter_3.keep_trace(&abcd_trace).unwrap());
    }

    #[rstest]
    fn test_no_matches_discards(abcd_trace: Trace) {
        let filter =
            FollowerFilter::new(vec!["y".to_string()], vec!["z".to_string()]).with_range(4); // 4 = Entire trace

        // Should discard the trace since this relation does not exist in the trace
        // since the activities "y" and "z" do not exist in the trace.
        assert!(!filter.keep_trace(&abcd_trace).unwrap());
    }

    #[rstest]
    #[case(0, 0)]
    #[case(1, 1)]
    #[case(2, 2)]
    #[case(3, 3)]
    #[case(4, 4)]
    #[case(5, 5)]
    #[case(6, 5)]
    #[case(7, 5)]
    fn simple_test(#[case] range: usize, #[case] expected: usize) {
        let mut filter =
            FollowerFilter::new(vec!["x".to_string()], vec!["y".to_string()]).with_range(range);
        let log = event_log!(
            ["x", "y"],                     // Range 1 (1 step)
            ["x", "a", "y"],                // Range 2
            ["x", "a", "b", "y"],           // Range 3
            ["x", "a", "b", "c", "y"],      // Range 4
            ["x", "a", "b", "c", "d", "y"], // Range 5
            ["x", "a", "b", "c", "d"],
            ["a", "b", "c", "d", "y"]
        );

        assert_eq!(expected, filter.apply(&log).unwrap().traces.len())
    }

    #[test]
    // An event does not eventually follow itself (i.e., we start "looking" for matches only at the
    // following index)
    fn irreflexive() {
        let filter =
            FollowerFilter::new(vec!["a".to_string()], vec!["a".to_string()]).with_range(0);
        assert!(!filter.keep_trace(&trace!("a", "b", "c", "d")).unwrap());

        // But: Identically labelled events _do_ count
        let filter =
            FollowerFilter::new(vec!["a".to_string()], vec!["a".to_string()]).with_range(2);
        assert!(filter.keep_trace(&trace!("a", "b", "a", "d")).unwrap());
    }
}
