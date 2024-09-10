use std::collections::HashSet;

use process_mining::{event_log::AttributeValue, EventLog};
use rand::{rngs::StdRng, seq::SliceRandom};

use crate::{
    constants::NO_TRACEID_MSG,
    utils::attributes::{get_traceid, set_traceid},
};

pub fn sample_log_without_replacement(rng: &mut StdRng, log: &EventLog, size: usize) -> EventLog {
    if size > log.traces.len() {
        panic!("Cannot sample without replacement with a size larger than the event log");
    }

    let mut new_log = log.clone();

    new_log.traces = log.traces.choose_multiple(rng, size).cloned().collect();
    new_log
}

pub fn sample_log_without_replacement_mut(rng: &mut StdRng, log: &mut EventLog, size: usize) {
    if size > log.traces.len() {
        panic!("Cannot sample without replacement with a size larger than the event log");
    }

    let retain_traceids: HashSet<String> = log
        .traces
        .choose_multiple(rng, size)
        .map(|trace| get_traceid(trace).expect(NO_TRACEID_MSG))
        .collect();
    log.traces
        .retain(|trace| retain_traceids.contains(&get_traceid(trace).expect(NO_TRACEID_MSG)));
}

pub fn sample_log_with_replacement(rng: &mut StdRng, log: &EventLog, size: usize) -> EventLog {
    let mut new_log = log.clone();
    // Sample `output_size` random cases
    new_log.traces = Vec::with_capacity(size);

    for i in 0..size {
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

pub fn sample_log_with_replacement_mut(rng: &mut StdRng, log: &mut EventLog, size: usize) {
    log.traces = (0..size)
        .map(|traceid| {
            let mut trace = log
                .traces
                .choose(rng)
                .expect("Cannot sample from an empty event log")
                .clone();
            set_traceid(&mut trace, AttributeValue::String(traceid.to_string()));
            trace
        })
        .collect();
}
