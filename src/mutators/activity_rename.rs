use process_mining::event_log::{AttributeValue, Event, Trace};
use rand::random;

use crate::{
    constants::NO_ACTIVITY_LABEL_MSG,
    mutation::TraceMutator,
    parsing::dir_name_trait::DirName,
    utils::{get_activity_label, set_activity_label},
};

#[derive(DirName)]
pub struct ActivityRenamer {
    /// The activity to rename. This modifier will only effect events with this label.
    #[dirname(rename = "from")]
    activity: String,
    /// The new activity label.
    #[dirname(rename = "to")]
    new_label: String,
    /// The probability of renaming. Ranges from 0 to 1.
    #[dirname(rename = "p", no_split)]
    probability: f32,
}

impl ActivityRenamer {
    pub fn new(activity: impl Into<String>, new_label: impl Into<String>) -> Self {
        Self {
            activity: activity.into(),
            new_label: new_label.into(),
            probability: 1.0,
        }
    }

    fn should_mutate(&self, event: &Event) -> bool {
        get_activity_label(event).expect(NO_ACTIVITY_LABEL_MSG) == self.activity
            && random::<f32>() < self.probability
    }

    pub fn with_probability(mut self, probability: f32) -> Self {
        self.probability = probability;
        self
    }
}

impl TraceMutator for ActivityRenamer {
    fn apply(&self, trace: &Trace) -> Trace {
        let mut new_trace = trace.clone();
        new_trace.events.iter_mut().for_each(|evt| {
            if self.should_mutate(evt) {
                set_activity_label(evt, AttributeValue::String(self.new_label.clone()));
            }
        });
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
    fn activity_renames_correctly(abcd_trace: Trace, #[case] activity: String) {
        let new_trace = ActivityRenamer::new(activity.clone(), "NEW_ACTIVITY").apply(&abcd_trace);

        let all_activities: Vec<_> = new_trace
            .events
            .iter()
            .map(|evt| get_activity_label(evt).unwrap())
            .collect();

        // The old activity is not contained
        assert!(!all_activities.contains(&activity));

        // The new activity is there now
        assert!(all_activities.contains(&"NEW_ACTIVITY".to_string()));

        // There are still 4 activities (Only the specified activity got renamed)
        // and since it is entirely gone, the renaming worked correctly
        assert_eq!(all_activities.len(), 4);
    }

    #[rstest]
    fn nonexistent_activity_doesnt_panic(abcd_trace: Trace) {
        // This should not panic
        let _ = ActivityRenamer::new("DOESNT_EXIST", "NEW_ACTIVITY").apply(&abcd_trace);
    }
}
