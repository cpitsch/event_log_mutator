use itertools::Itertools;
use process_mining::event_log::{AttributeValue, Trace};
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    mutation::{MutationError, MutationResult, TraceMutator},
    parsing::traits::DirName,
    utils::attributes::{
        get_activity_label, get_service_time, get_start_timestamp, set_complete_timestamp,
        set_start_timestamp, AttributeResult,
    },
};

/// Swap pairs of events with activity label `activity_1` and `activity_2`.
/// First occurrence of `activity_1` is swapped with first occurrence of `activity_2`,
/// etc.
/// Un-paired events are not affected.
#[derive(DirName)]
pub struct EventSwapper {
    // The first activity label for swapping.
    #[dirname(rename = "")]
    activity_1: String,
    // The second activity label for swapping.
    #[dirname(rename = "swap")]
    activity_2: String,
    /// The probability of applying this modifier (per pair). Use
    /// [`EventSwapper::with_probability`] to set the probability.
    #[dirname(rename = "p", no_split)]
    probability: f32,
    /// Optional seed for the random number generator. Ensures reproducible results
    /// across runs. Use [`EventSwapper::with_seed`] to set the seed.
    seed: Option<u64>,
    #[dirname(ignore)]
    rng: StdRng,
}

impl EventSwapper {
    pub fn new(activity_1: impl Into<String>, activity_2: impl Into<String>) -> Self {
        Self {
            activity_1: activity_1.into(),
            activity_2: activity_2.into(),
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

impl TraceMutator for EventSwapper {
    fn apply_mut(&mut self, trace: &mut Trace) -> MutationResult<()> {
        // Get all indices of activity 1 and 2
        let activities: Vec<_> = trace
            .events
            .iter()
            .map(get_activity_label)
            .collect::<AttributeResult<Vec<_>>>()
            .map_err(|e| MutationError::AttributeError("EventSwapper", e))?;
        let act_1_indices = activities
            .iter()
            .positions(|act| **act == self.activity_1)
            .collect_vec();

        let act_2_indices = activities
            .into_iter()
            .positions(|act| *act == self.activity_2)
            .collect_vec();

        for (idx_1, idx_2) in act_1_indices.iter().zip(act_2_indices.iter()) {
            if self.should_mutate() {
                // Swap their start_timestamp, and update their complete timestamp
                // based on their service time
                let event_1_start = *get_start_timestamp(trace.events.get(*idx_1).unwrap())
                    .map_err(|e| MutationError::AttributeError("EventSwapper", e))?;
                let event_2_start = *get_start_timestamp(trace.events.get(*idx_2).unwrap())
                    .map_err(|e| MutationError::AttributeError("EventSwapper", e))?;

                let evt_1_service_time = get_service_time(trace.events.get(*idx_1).unwrap())
                    .map_err(|e| MutationError::AttributeError("EventSwapper", e))?;
                let evt_2_service_time = get_service_time(trace.events.get(*idx_2).unwrap())
                    .map_err(|e| MutationError::AttributeError("EventSwapper", e))?;

                // Swap start timestamps
                set_start_timestamp(
                    trace.events.get_mut(*idx_1).unwrap(),
                    AttributeValue::Date(event_2_start),
                );
                set_start_timestamp(
                    trace.events.get_mut(*idx_2).unwrap(),
                    AttributeValue::Date(event_1_start),
                );
                // Update complete timestamps to match old service time
                set_complete_timestamp(
                    trace.events.get_mut(*idx_1).unwrap(),
                    AttributeValue::Date(event_2_start + evt_1_service_time),
                );
                set_complete_timestamp(
                    trace.events.get_mut(*idx_2).unwrap(),
                    AttributeValue::Date(event_1_start + evt_2_service_time),
                );

                // Swap them in the trace events vec
                trace.events.swap(*idx_1, *idx_2);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test_fixtures::{abcd_trace, get_control_flow},
        utils::attributes::get_complete_timestamp,
    };
    use rstest::rstest;

    #[rstest]
    #[case::swap_a_d("a", "d", "dbca")]
    #[case::swap_b_c("b", "c", "acbd")]
    #[case::swap_b_c("b", "d", "adcb")]
    // Since activity_2 doesn't exist, it stays the same
    #[case::swap_a_non_existent("a", "NONEXISTENT", "abcd")]
    fn event_swapper_swaps_correctly_and_updates_times(
        abcd_trace: Trace,
        #[case] activity_1: String,
        #[case] activity_2: String,
        #[case] expected: String,
    ) {
        let new_trace = EventSwapper::new(activity_1, activity_2)
            .apply(&abcd_trace)
            .unwrap();

        assert_eq!(
            expected,
            new_trace
                .events
                .iter()
                .map(|evt| get_activity_label(evt).unwrap())
                .join("")
        );

        // Sort by timestamp to make sure that the timestamps were updated correctly
        assert_eq!(
            expected,
            new_trace
                .events
                .iter()
                .sorted_by_key(|evt| get_complete_timestamp(evt).unwrap())
                .map(|evt| get_activity_label(evt).unwrap())
                .join("")
        );
    }

    #[rstest]
    fn seeded_gives_same_result(abcd_trace: Trace) {
        for _ in 1..1000 {
            let new_trace_1 = EventSwapper::new("b", "c")
                .with_probability(0.5)
                .with_seed(42)
                .apply(&abcd_trace)
                .unwrap();
            let new_trace_2 = EventSwapper::new("b", "c")
                .with_probability(0.5)
                .with_seed(42)
                .apply(&abcd_trace)
                .unwrap();

            assert_eq!(
                get_control_flow(&new_trace_1),
                get_control_flow(&new_trace_2)
            )
        }
    }
}
