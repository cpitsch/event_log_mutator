use std::collections::HashMap;

use chrono::{SubsecRound, TimeDelta};
use process_mining::{
    event_log::{AttributeValue, Event, Trace},
    EventLog,
};
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    constants::{NO_ACTIVITY_LABEL_MSG, NO_COMPLETE_TIMESTAMP_MSG, NO_START_TIMESTAMP_MSG},
    mutation::LogMutator,
    parsing::dir_name_trait::DirName,
    utils::{
        get_activity_label, get_complete_timestamp, get_service_time, get_start_timestamp,
        set_complete_timestamp, set_start_timestamp,
    },
};

/// Mutation to shift the execution times of events by a factor of the
/// standard deviation of the duration
#[derive(DirName)]
pub struct ServiceTimeStdShifter {
    /// Only mutate events with this activity. Defaults to all activities (None).
    /// Use [`ServiceTimeStdShifter::for_activity`] to for a specific activity.
    #[dirname(rename = "")]
    activity: Option<String>,
    /// The number of standard deviations to shift the duration by.
    #[dirname(rename = "std", no_split)]
    standard_deviations: f64,
    /// The probability to apply the mutation to a matching event. Ranges from 0 to 1.
    /// Use [`ServiceTimeStdShifter::with_probability`] for a specific probability.
    #[dirname(rename = "p", no_split)]
    probability: f32,
    /// Optional seed for the random number generator. Ensures reproducible results
    /// across runs. Use [`ServiceTimeStdShifter::with_seed`] to set the seed.
    seed: Option<u64>,
    #[dirname(ignore)]
    rng: StdRng,
}

impl ServiceTimeStdShifter {
    pub fn new(standard_deviations: f64) -> Self {
        Self {
            activity: None,
            probability: 1.0,
            standard_deviations,
            seed: None,
            rng: StdRng::from_entropy(),
        }
    }

    fn should_mutate(&mut self, event: &Event) -> bool {
        (
            // Check that the event matches the requirements
            self.activity.as_ref().map_or(true, |act| {
                get_activity_label(event).expect(NO_ACTIVITY_LABEL_MSG) == *act
            })
        ) && (
            // Check mutation probability
            self.rng.gen::<f32>() < self.probability
        )
    }

    pub fn for_activity(mut self, activity: impl Into<String>) -> Self {
        self.activity = Some(activity.into());
        self
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

    /// Apply the service time mutation to an event. Note: this does _not_ check
    /// `self.should_mutate(evt)`, as this is done by [`apply_trace`].
    fn mutate_event(&self, evt: &mut Event, shift_amounts: &HashMap<String, chrono::TimeDelta>) {
        let activity = get_activity_label(evt).expect(NO_ACTIVITY_LABEL_MSG);
        let start_timestamp = get_start_timestamp(evt).expect(NO_START_TIMESTAMP_MSG);
        let service_time = get_service_time(evt).expect(NO_COMPLETE_TIMESTAMP_MSG);
        let increment = shift_amounts
            .get(&activity)
            .cloned()
            .unwrap_or(chrono::TimeDelta::seconds(0));
        let new_serivce_time = service_time + increment;

        set_complete_timestamp(
            evt,
            // Round duration seconds to 6 decimal places so pm4py imports it correctly
            AttributeValue::Date((start_timestamp + new_serivce_time).round_subsecs(6)),
        );
    }

    /// Apply the service time mutation to an event. Checks `self.should_mutate(evt)`.
    /// Also shifts the following events by the service time increment.
    fn mutate_trace(
        &mut self,
        trace: &mut Trace,
        shift_amounts: &HashMap<String, chrono::TimeDelta>,
    ) {
        let mut shift_amount = TimeDelta::zero();

        trace.events.iter_mut().for_each(|event| {
            let start_timestamp = get_start_timestamp(event).expect(NO_START_TIMESTAMP_MSG);
            let complete_timestamp =
                get_complete_timestamp(event).expect(NO_COMPLETE_TIMESTAMP_MSG);

            if !shift_amount.is_zero() {
                set_start_timestamp(event, AttributeValue::Date(start_timestamp + shift_amount));
                set_complete_timestamp(
                    event,
                    AttributeValue::Date(complete_timestamp + shift_amount),
                );
            }

            if self.should_mutate(event) {
                self.mutate_event(event, shift_amounts);
                let shifted_by = get_complete_timestamp(event).expect(NO_COMPLETE_TIMESTAMP_MSG)
                    - complete_timestamp;

                // Need to move all following events if changed service time
                // Otherwise, we induce control-flow changes that are unwanted
                // side-effects from this mutation
                shift_amount += shifted_by;
            }
        });
    }
}

/// Compute the standard deviation of timedeltas.
///
/// Converts the timedeltas to nanoseconds, computes the standard deviation on
/// this, converts the floor of this value back to a timedelta.
///
/// Panics if the conversion of a timedelta to nanoseconds fails. This only occurs
/// if the timedelta is over 2^63ns, so ~292.5yrs.
fn timedelta_standard_deviation(timedeltas: Vec<chrono::TimeDelta>) -> chrono::TimeDelta {
    let milliseconds: Vec<i64> = timedeltas
        .iter()
        .map(|td| {
            td.num_milliseconds()
            // .expect("Attempt to convert a timedelta over 292.5yrs (2^63ns) to nanoseconds.")
        })
        .collect();
    let standard_deviation_ms = std(milliseconds);
    chrono::TimeDelta::milliseconds(standard_deviation_ms.floor() as i64)
}

/// Compute the standard deviation for a vec of i64. Computes the population
/// standard deviation, i.e., divides by n-1
/// Panics if conversion of vec len to i64 fails.
fn std(values: Vec<i64>) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let len: i64 = values.len().try_into().unwrap();

    let sum: i64 = values.iter().sum();
    let mean: f64 = (sum as f64) / len as f64;
    let sum_of_squares: f64 = values.iter().map(|&x| ((x as f64) - mean).powi(2)).sum();

    let variance = sum_of_squares / ((values.len() - 1) as f64);
    variance.sqrt()
}

/// Helper function to multiply a timedelta by a float.
///
/// - Try to multiply nanoseconds by the float, then round ns and convert back to TimeDelta.
/// - If TimeDelta spans more than 2^63 nanoseconds (292.5 yrs), work on seconds instead.
fn multiply_timedelta_by_float(timedelta: TimeDelta, factor: &f64) -> TimeDelta {
    // Can't multiply TimeDelta by float, so need to be a bit less exact

    // num_nanoseconds only returns None if Timedelta > ~295.2yrs
    if let Some(ns) = timedelta.num_nanoseconds() {
        let new_ns = (ns as f64) * *factor;
        TimeDelta::nanoseconds(new_ns.floor() as i64)
    } else {
        let seconds = timedelta.num_seconds() as f64;
        let new_seconds = seconds * *factor;
        TimeDelta::seconds(new_seconds.round() as i64)
    }
}

/// Compute a hashmap mapping each activity in the event log to a vec of all its
/// durations.
fn get_activity_durations(log: &EventLog) -> HashMap<String, Vec<TimeDelta>> {
    let mut res: HashMap<String, Vec<TimeDelta>> = HashMap::new();

    log.traces
        .iter()
        .flat_map(|trace| trace.events.iter())
        .for_each(|evt| {
            let activity = get_activity_label(evt).expect(NO_ACTIVITY_LABEL_MSG);
            let service_time = get_service_time(evt).expect(NO_START_TIMESTAMP_MSG);

            res.entry(activity).or_default().push(service_time);
        });
    res
}

// Compute a hashmap mapping each activity in the event log to the standard deviation
// of its duration
fn get_activity_duration_stds(log: &EventLog) -> HashMap<String, chrono::TimeDelta> {
    let durations = get_activity_durations(log);
    let res: HashMap<_, _> = durations
        .into_iter()
        .map(|(act, durs)| (act, timedelta_standard_deviation(durs)))
        .collect();
    res
}

impl LogMutator for ServiceTimeStdShifter {
    fn apply_mut(&mut self, log: &mut EventLog) {
        // First collect the duration for each activity
        let stds = get_activity_duration_stds(log);
        let shift_amounts: HashMap<String, chrono::TimeDelta> = stds
            .into_iter()
            .map(|(act, std)| {
                (
                    act,
                    multiply_timedelta_by_float(std, &self.standard_deviations),
                )
            })
            .collect();

        log.traces
            .iter_mut()
            .for_each(|trace| self.mutate_trace(trace, &shift_amounts));
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use rstest::{fixture, rstest};

    use super::*;

    use crate::test_fixtures::{log_from_traces, new_event};

    fn test_trace_1() -> Trace {
        let date = Utc
            .with_ymd_and_hms(2024, 4, 29, 1, 0, 0)
            .earliest()
            .unwrap();
        Trace {
            attributes: Vec::default(),
            events: vec![
                new_event("a", date, TimeDelta::hours(1)),
                // Starts exactly as the previous finishes.
                new_event("b", date + TimeDelta::hours(3), TimeDelta::hours(2)),
            ],
        }
    }

    fn test_trace_2() -> Trace {
        let date = Utc
            .with_ymd_and_hms(2024, 4, 29, 1, 0, 0)
            .earliest()
            .unwrap();
        Trace {
            attributes: Vec::default(),
            events: vec![
                new_event("b", date, TimeDelta::hours(4)),
                new_event("b", date, TimeDelta::hours(3)),
            ],
        }
    }

    #[fixture]
    fn test_log() -> EventLog {
        log_from_traces(vec![test_trace_1(), test_trace_2()])
    }

    #[rstest]
    fn correct_duration_extraction(test_log: EventLog) {
        let durations = get_activity_durations(&test_log);

        assert_eq!(
            vec![TimeDelta::hours(1)],
            durations.get("a").unwrap().clone()
        );

        assert_eq!(
            vec![
                TimeDelta::hours(2),
                TimeDelta::hours(4),
                TimeDelta::hours(3)
            ],
            durations.get("b").unwrap().clone()
        );
    }

    #[rstest]
    fn correct_std_computation(test_log: EventLog) {
        let stds = get_activity_duration_stds(&test_log);

        assert_eq!(TimeDelta::nanoseconds(0), stds.get("a").unwrap().clone());

        // 2, 3, 4 --> mean 3, squared deviations of 1, 0 ,1
        // So we get sqrt(2/(n-1)) = sqrt(2/2) = 1
        assert_eq!(TimeDelta::hours(1), stds.get("b").unwrap().clone());
    }

    #[rstest]
    fn applies_correctly(test_log: EventLog) {
        let new_log = ServiceTimeStdShifter::new(2.5).apply(&test_log);

        let new_durations = get_activity_durations(&new_log);
        assert_eq!(
            vec![TimeDelta::hours(1)], // "a" had a standard deviation of 0h, so unchanged
            new_durations.get("a").unwrap().clone()
        );

        assert_eq!(
            vec![
                // "b" had a standard deviation of 1h, so add 2.5*1h everywhere
                TimeDelta::minutes((2 * 60) + 150),
                TimeDelta::minutes((4 * 60) + 150),
                TimeDelta::minutes((3 * 60) + 150)
            ],
            new_durations.get("b").unwrap().clone()
        );
    }

    #[rstest]
    fn seeded_gives_same_result(test_log: EventLog) {
        let new_log_1 = ServiceTimeStdShifter::new(2.0)
            .for_activity("a")
            .with_probability(0.5)
            .with_seed(42)
            .apply(&test_log);

        let new_log_2 = ServiceTimeStdShifter::new(2.0)
            .for_activity("a")
            .with_probability(0.5)
            .with_seed(42)
            .apply(&test_log);

        let log_1_service_times: Vec<Vec<_>> = new_log_1
            .traces
            .iter()
            .map(|trace| {
                trace
                    .events
                    .iter()
                    .map(|evt| get_service_time(evt).unwrap())
                    .collect()
            })
            .collect();

        let log_2_service_times: Vec<Vec<_>> = new_log_2
            .traces
            .iter()
            .map(|trace| {
                trace
                    .events
                    .iter()
                    .map(|evt| get_service_time(evt).unwrap())
                    .collect()
            })
            .collect();

        assert_eq!(log_1_service_times, log_2_service_times);
    }
}
