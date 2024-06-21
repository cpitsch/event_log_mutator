pub const TRACEID_KEY: &str = "concept:name";
pub const ACTIVITY_KEY: &str = "concept:name";
pub const START_TIMESTAMP_KEY: &str = "start_timestamp";
pub const TIMESTAMP_KEY: &str = "time:timestamp";

// Error Messages
pub const NO_ACTIVITY_LABEL_MSG: &str = "All events must have an activity label (\"concept:name\")";
pub const NO_START_TIMESTAMP_MSG: &str =
    "All events must have an activity label (\"start_timestamp\")";
pub const NO_COMPLETE_TIMESTAMP_MSG: &str =
    "All events must have an activity label (\"time:timestamp\")";
pub const NO_TRACEID_MSG: &str = "All traces must have a trace id (\"concept:name\")";
