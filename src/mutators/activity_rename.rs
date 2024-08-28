use process_mining::event_log::{AttributeValue, Event, Trace};
use rand::{rngs::StdRng, Rng, SeedableRng};

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
    /// Optional seed for the random number generator. Ensures reproducible results
    /// across runs.
    seed: Option<u64>,
    #[dirname(ignore)]
    rng: StdRng,
}

impl ActivityRenamer {
    pub fn new(activity: impl Into<String>, new_label: impl Into<String>) -> Self {
        Self {
            activity: activity.into(),
            new_label: new_label.into(),
            probability: 1.0,
            seed: None,
            rng: StdRng::from_entropy(),
        }
    }

    fn should_mutate(&mut self, event: &Event) -> bool {
        get_activity_label(event).expect(NO_ACTIVITY_LABEL_MSG) == self.activity
            && self.rng.gen::<f32>() < self.probability
    }

    pub fn with_probability(mut self, probability: f32) -> Self {
        self.probability = probability;
        self
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self.rng = StdRng::seed_from_u64(seed);
        self
    }
}

impl TraceMutator for ActivityRenamer {
    fn apply(&mut self, trace: &Trace) -> Trace {
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
    use crate::test_fixtures::{abcd_trace, get_control_flow};
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

    #[rstest]
    fn seeded_gives_same_result(abcd_trace: Trace) {
        for _ in 1..1000 {
            let new_trace_1 = ActivityRenamer::new("b", "NEW_B")
                .with_probability(0.5)
                .with_seed(42)
                .apply(&abcd_trace);
            let new_trace_2 = ActivityRenamer::new("b", "NEW_B")
                .with_probability(0.5)
                .with_seed(42)
                .apply(&abcd_trace);

            assert_eq!(
                get_control_flow(&new_trace_1),
                get_control_flow(&new_trace_2)
            )
        }
    }
}
