use process_mining::{event_log::AttributeValue, EventLog};
use rand::seq::SliceRandom;

use crate::{mutation::LogMutator, parsing::as_dir_name::AsDirName, utils::set_traceid};

/// Mutator to create a new log by randomly sampling cases with replacement.
/// The sampled cases are assigned unique case ids ("0" ... "`size`").
#[derive(AsDirName)]
pub struct LogBootstrapper {
    /// The number of cases to sample.
    #[asdirname(rename = "")]
    size: usize,
    /// Sample with replacement? Defaults to true.
    replacement: bool,
}

impl LogBootstrapper {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            replacement: true,
        }
    }
}

impl LogMutator for LogBootstrapper {
    fn apply(&self, log: &EventLog) -> EventLog {
        if self.replacement {
            self.sample_with_replacement(log)
        } else {
            self.sample_without_replacement(log)
        }
    }
}

impl LogBootstrapper {
    pub fn with_replacement(mut self, replacement: bool) -> Self {
        self.replacement = replacement;
        self
    }

    fn sample_with_replacement(&self, log: &EventLog) -> EventLog {
        let mut new_log = log.clone();
        // Sample `output_size` random cases
        let rng = &mut rand::thread_rng();
        new_log.traces = Vec::with_capacity(self.size);

        for i in 0..self.size {
            let mut new_trace = log
                .traces
                .choose(rng)
                .expect("Cannot bootstrap an empty event log.")
                .clone();

            set_traceid(&mut new_trace, AttributeValue::String(i.to_string()));

            new_log.traces.push(new_trace);
        }

        new_log
    }

    fn sample_without_replacement(&self, log: &EventLog) -> EventLog {
        if self.size > log.traces.len() {
            panic!("Cannot sample without replacement with a size larger than the event log");
        }

        let mut new_log = log.clone();

        // Sample `output_size` random cases
        let rng = &mut rand::thread_rng();
        new_log.traces = log
            .traces
            .choose_multiple(rng, self.size)
            .cloned()
            .collect();
        new_log
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use chrono::{DateTime, TimeDelta, TimeZone, Utc};
    use process_mining::{
        event_log::{Attribute, AttributeValue, Event, Trace},
        EventLog,
    };

    use crate::{
        mutation::LogMutator,
        utils::{get_string_by_key, get_traceid, get_traceids},
    };

    use super::LogBootstrapper;

    fn new_event(
        activity: impl Into<String>,
        start_timestamp: DateTime<Utc>,
        service_time: TimeDelta,
    ) -> Event {
        Event {
            attributes: vec![
                Attribute::new(
                    "concept:name".to_owned(),
                    AttributeValue::String(activity.into()),
                ),
                Attribute::new(
                    "start_timestamp".to_owned(),
                    AttributeValue::Date(start_timestamp),
                ),
                Attribute::new(
                    "time:timestamp".to_owned(),
                    AttributeValue::Date(start_timestamp + service_time),
                ),
            ],
        }
    }

    fn test_log() -> EventLog {
        let date = Utc
            .with_ymd_and_hms(2024, 4, 29, 1, 0, 0)
            .earliest()
            .unwrap();

        EventLog {
            attributes: Vec::default(),
            traces: vec![
                Trace {
                    attributes: vec![Attribute {
                        key: "concept:name".to_string(),
                        value: AttributeValue::String("1".to_string()),
                        own_attributes: None,
                    }],
                    events: vec![new_event("a", date, TimeDelta::hours(1))],
                },
                Trace {
                    attributes: vec![Attribute {
                        key: "concept:name".to_string(),
                        value: AttributeValue::String("2".to_string()),
                        own_attributes: None,
                    }],
                    events: vec![new_event("b", date, TimeDelta::hours(1))],
                },
                Trace {
                    attributes: vec![Attribute {
                        key: "concept:name".to_string(),
                        value: AttributeValue::String("3".to_string()),
                        own_attributes: None,
                    }],
                    events: vec![new_event("c", date, TimeDelta::hours(1))],
                },
                Trace {
                    attributes: vec![Attribute {
                        key: "concept:name".to_string(),
                        value: AttributeValue::String("4".to_string()),
                        own_attributes: None,
                    }],
                    events: vec![new_event("d", date, TimeDelta::hours(1))],
                },
            ],
            extensions: None,
            classifiers: None,
            global_trace_attrs: None,
            global_event_attrs: None,
        }
    }

    #[test]
    #[should_panic]
    fn sample_without_replacement_fails_with_large_size() {
        let test_log = test_log();
        LogBootstrapper::new(5)
            .with_replacement(false)
            .apply(&test_log);
    }

    #[test]
    fn sample_without_replacement_has_no_duplicates() {
        let test_log = test_log();
        let trace_ids = get_traceids(&test_log);
        // Do it a couple of times to make (more) sure that we aren't getting lucky
        for _ in 1..10 {
            let mutated_log = LogBootstrapper::new(4)
                .with_replacement(false)
                .apply(&test_log);
            let new_traceids = get_traceids(&mutated_log);

            assert_eq!(trace_ids, new_traceids);
        }
    }

    #[test]
    fn sample_without_replacement_is_random() {
        let test_log = test_log();
        let mut seen_trace_ids: HashSet<String> = HashSet::new();

        // Test that sampling multiple times yields different results.
        for _ in 1..10 {
            let mutated_log = LogBootstrapper::new(1)
                .with_replacement(false)
                .apply(&test_log);
            seen_trace_ids = seen_trace_ids
                .union(&get_traceids(&mutated_log))
                .cloned()
                .collect();
        }

        assert!(seen_trace_ids.len() > 1);
    }

    #[test]
    fn sample_with_replacement_has_duplicates() {
        let mut test_log = test_log();

        test_log.traces.iter_mut().for_each(|trace| {
            let traceid = get_traceid(trace).unwrap();
            trace.attributes.push(Attribute {
                key: "original_traceid".to_string(),
                value: AttributeValue::String(traceid),
                own_attributes: None,
            });
        });

        // Don't explicitly specify the
        let mutated_log = LogBootstrapper::new(1000)
            .with_replacement(true)
            .apply(&test_log);

        let mut traceids: Vec<String> = mutated_log
            .traces
            .iter()
            .map(|trace| get_string_by_key(trace, "original_traceid").unwrap())
            .collect();
        traceids.sort();

        let has_dups = traceids.windows(2).any(|window| window[0] == window[1]);

        assert!(has_dups);
    }

    #[test]
    fn default_is_with_replacement() {
        let mutator = LogBootstrapper::new(10);
        assert!(mutator.replacement);
    }
}
