use chrono::{DateTime, FixedOffset, TimeDelta, TimeZone, Utc};
use process_mining::core::event_data::case_centric::{
    Attribute, AttributeValue, Event, EventLog, Trace,
};
use rstest::fixture;

use crate::utils::attributes::get_activity_label;

pub fn new_event(
    activity: impl Into<String>,
    start_timestamp: DateTime<FixedOffset>,
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

pub fn log_from_traces(traces: Vec<Trace>) -> EventLog {
    EventLog {
        attributes: Vec::new(),
        traces,
        extensions: None,
        classifiers: None,
        global_trace_attrs: None,
        global_event_attrs: None,
    }
}

pub fn get_control_flow(trace: &Trace) -> Vec<String> {
    trace
        .events
        .iter()
        .map(|evt| get_activity_label(evt).unwrap().clone())
        .collect()
}

#[fixture]
pub fn abcd_trace() -> Trace {
    let date = Utc
        .with_ymd_and_hms(2024, 4, 29, 1, 0, 0)
        .earliest()
        .unwrap()
        .fixed_offset();
    Trace {
        attributes: Vec::default(),
        events: vec![
            new_event("a", date, TimeDelta::hours(1)),
            new_event("b", date + TimeDelta::hours(1), TimeDelta::hours(2)),
            new_event("c", date + TimeDelta::hours(3), TimeDelta::hours(2)),
            new_event("d", date + TimeDelta::hours(5), TimeDelta::hours(2)),
        ],
    }
}

#[fixture]
pub fn abcd_log() -> EventLog {
    let date = Utc
        .with_ymd_and_hms(2024, 4, 29, 1, 0, 0)
        .earliest()
        .unwrap()
        .fixed_offset();

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

/// A toml file containing only the required fields for all mutators
#[fixture]
pub fn mininmal_toml_example() -> &'static str {
    return "
input = \"input_log.xes.gz\"

[pipeline]
[[pipeline.mutations]]
type = \"ServiceTimeStdShifter\"
standard_deviations = 1.0

[[pipeline.mutations]]
type = \"VariantSupportFilter\"
num_supporting_cases = 25

[[pipeline.mutations]]
type = \"EndpointFilter\"

[[pipeline.mutations]]
type = \"CaseDurationFilter\"

[[pipeline.mutations]]
type = \"ActivityRemover\"
activity = \"a\"

[[pipeline.mutations]]
type = \"ActivityRenamer\"
activity = \"a\"
new_label = \"a'\"

[[pipeline.mutations]]
type = \"ConstantActivity\"
activity = \"new_activity\"

[[pipeline.mutations]]
type = \"EventSwapper\"
activity_1 = \"b\"
activity_2 = \"c\"

[[pipeline.mutations]]
type = \"LogBootstrapper\"
size = 2000

[[pipeline.mutations]]
type = \"PartialOrderCreator\"

[[pipeline.mutations]]
type = \"AttributeRemover\"
key = \"start_timestamp\"

[[pipeline.mutations]]
type = \"ServiceTimeMultiplier\"
";
}
