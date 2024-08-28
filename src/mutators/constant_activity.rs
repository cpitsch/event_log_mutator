use process_mining::event_log::{AttributeValue, Event};
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{mutation::EventMutator, parsing::dir_name_trait::DirName, utils::set_activity_label};

/// Replace the activity label of all events with a constant one.
#[derive(DirName)]
pub struct ConstantActivityMutator {
    /// The activity label to use.
    #[dirname(rename = "")]
    activity: String,
    /// The probability of applying the mutation to an event. Use
    /// [`ConstantActivityMutator::with_probability`] to set the probability.
    #[dirname(rename = "p", no_split)]
    probability: f32,
    /// Optional seed for the random number generator. Ensures reproducible results
    /// across runs. Use [`ConstantActivityMutator::with_seed`] to set the seed.
    seed: Option<u64>,
    #[dirname(ignore)]
    rng: StdRng,
}

impl ConstantActivityMutator {
    pub fn new(activity: impl Into<String>) -> Self {
        Self {
            activity: activity.into(),
            probability: 1.0,
            seed: None,
            rng: StdRng::from_entropy(),
        }
    }

    fn should_mutate(&mut self) -> bool {
        self.rng.gen::<f32>() < self.probability
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

impl EventMutator for ConstantActivityMutator {
    fn apply(&mut self, evt: &Event) -> Event {
        if self.should_mutate() {
            let mut new_event = evt.clone();

            set_activity_label(
                &mut new_event,
                AttributeValue::String(self.activity.clone()),
            );

            new_event
        } else {
            evt.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ConstantActivityMutator;
    use crate::{
        mutation::TraceMutator,
        test_fixtures::{abcd_trace, get_control_flow},
        utils::get_activity_label,
    };
    use process_mining::event_log::Trace;
    use rstest::rstest;

    #[rstest]
    fn all_events_rename(abcd_trace: Trace) {
        let new_trace = ConstantActivityMutator::new("NEW_ACTIVITY".to_string()).apply(&abcd_trace);

        assert!(new_trace
            .events
            .iter()
            .all(|evt| get_activity_label(evt).unwrap() == *"NEW_ACTIVITY"));
    }

    #[rstest]
    fn seeded_gives_same_result(abcd_trace: Trace) {
        for _ in 1..1000 {
            let new_trace_1 = ConstantActivityMutator::new("New Activity")
                .with_probability(0.5)
                .with_seed(42)
                .apply(&abcd_trace);
            let new_trace_2 = ConstantActivityMutator::new("New Activity")
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
