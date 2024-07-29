use process_mining::event_log::{Event, Trace};
use rand::random;

use crate::{
    constants::NO_ACTIVITY_LABEL_MSG, mutation::TraceMutator, parsing::dir_name_trait::DirName,
    utils::get_activity_label,
};

/// Mutator to remove events that have the given activity label.
#[derive(DirName)]
pub struct ActivityRemover {
    /// The activity label to remove.
    #[dirname(rename = "")]
    activity: String,
    /// The probability of removal. Ranges from 0 to 1. Defaults to 1
    #[dirname(rename = "p", no_split)]
    probability: f32,
}

impl ActivityRemover {
    pub fn new(activity: impl Into<String>) -> Self {
        Self {
            activity: activity.into(),
            probability: 1.0,
        }
    }

    fn should_remove(&self, event: &Event) -> bool {
        get_activity_label(event).expect(NO_ACTIVITY_LABEL_MSG) == self.activity
            && random::<f32>() < self.probability
    }

    pub fn with_probability(mut self, probability: f32) -> Self {
        self.probability = probability;
        self
    }
}

impl TraceMutator for ActivityRemover {
    fn apply(&self, trace: &Trace) -> Trace {
        let mut new_trace = trace.clone();
        new_trace.events.retain(|evt| !self.should_remove(evt));
        new_trace
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::abcd_trace;
    use rstest::rstest;

    #[rstest]
    #[case::remove_a("a")]
    #[case::remove_b("b")]
    #[case::remove_c("c")]
    #[case::remove_d("d")]
    fn activity_removes_rest_remains(abcd_trace: Trace, #[case] activity: String) {
        let new_trace = ActivityRemover::new(activity.clone()).apply(&abcd_trace);

        let all_activities: Vec<_> = new_trace
            .events
            .iter()
            .map(|evt| get_activity_label(evt).unwrap())
            .collect();

        // One of the 4 activities is removed, the rest stays
        assert_eq!(all_activities.len(), 3);

        // This activity is not contained
        assert!(!all_activities.contains(&activity));
    }

    #[rstest]
    fn nonexistent_activity_doesnt_panic(abcd_trace: Trace) {
        // This should not panic
        let _ = ActivityRemover::new("DOESNT_EXIST").apply(&abcd_trace);
    }
}
