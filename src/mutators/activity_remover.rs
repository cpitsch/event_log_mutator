use process_mining::event_log::{Event, Trace};
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    constants::NO_ACTIVITY_LABEL_MSG, mutation::TraceMutator, parsing::dir_name_trait::DirName,
    utils::attributes::get_activity_label,
};

/// Mutator to remove events that have the given activity label.
#[derive(DirName)]
pub struct ActivityRemover {
    /// The activity label to remove.
    #[dirname(rename = "")]
    activity: String,
    /// The probability of removal. Ranges from 0 to 1. Defaults to 1. use
    /// [`ActivityRemover::with_probability`] to set the probability.
    #[dirname(rename = "p", no_split)]
    probability: f32,
    /// Optional seed for the random number generator. Ensures reproducible results
    /// across runs. Use [`ActivityRemover::with_seed`] to set the seed.
    seed: Option<u64>,
    #[dirname(ignore)]
    rng: StdRng,
}

impl ActivityRemover {
    pub fn new(activity: impl Into<String>) -> Self {
        Self {
            activity: activity.into(),
            probability: 1.0,
            seed: None,
            rng: StdRng::from_entropy(),
        }
    }

    fn should_remove(&mut self, event: &Event) -> bool {
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

impl TraceMutator for ActivityRemover {
    fn apply_mut(&mut self, trace: &mut Trace) {
        trace.events.retain(|evt| !self.should_remove(evt));
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

    #[rstest]
    fn seeded_gives_same_result(abcd_trace: Trace) {
        for _ in 1..1000 {
            let new_trace_1 = ActivityRemover::new("b")
                .with_probability(0.5)
                .with_seed(42)
                .apply(&abcd_trace);
            let new_trace_2 = ActivityRemover::new("b")
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
