use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use process_mining::event_log::{Attribute, AttributeValue, Event, EventLog, Trace};
use rstest::fixture;

pub fn new_event(
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

#[fixture]
pub fn abcd_trace() -> Trace {
    let date = Utc
        .with_ymd_and_hms(2024, 4, 29, 1, 0, 0)
        .earliest()
        .unwrap();
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
