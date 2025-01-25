use process_mining::EventLog;

// Could also implement a streamed comparison. So would take an event log and a path
// And stream the loading of the event log and compare traces on the fly. This would
// reduce memory usage a bit
pub fn event_logs_are_identical(log_1: &EventLog, log_2: &EventLog) -> bool {
    log_1 == log_2
}
