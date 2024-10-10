use std::collections::HashSet;

use process_mining::{event_log::AttributeValue, EventLog};
use rand::{rngs::StdRng, seq::SliceRandom};

use crate::{
    mutation::{MutationError, MutationResult},
    utils::attributes::{get_traceid, set_traceid},
};

use super::attributes::AttributeResult;

pub fn sample_log_without_replacement(
    rng: &mut StdRng,
    log: &EventLog,
    size: usize,
) -> MutationResult<EventLog> {
    if size > log.traces.len() {
        return Err(MutationError::InvalidValue(format!(
            "Cannot sample without replacement with a size larger than the event log ({}>{})",
            size,
            log.traces.len()
        )));
    }

    let mut new_log = log.clone();

    new_log.traces = log.traces.choose_multiple(rng, size).cloned().collect();
    Ok(new_log)
}

pub fn sample_log_without_replacement_mut(
    rng: &mut StdRng,
    log: &mut EventLog,
    size: usize,
) -> MutationResult<()> {
    if size > log.traces.len() {
        return Err(MutationError::InvalidValue(format!(
            "Cannot sample without replacement with a size larger than the event log ({}>{})",
            size,
            log.traces.len()
        )));
    }

    let retain_traceids: HashSet<String> = log
        .traces
        .choose_multiple(rng, size)
        .map(get_traceid)
        .collect::<AttributeResult<_>>()
        .map_err(|e| MutationError::MissingAttributeError("SampleWithoutReplacement", e))?;
    log.traces
        .retain(|trace| retain_traceids.contains(&get_traceid(trace).unwrap()));
    Ok(())
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

pub fn sample_log_with_replacement_mut(
    rng: &mut StdRng,
    log: &mut EventLog,
    size: usize,
) -> MutationResult<()> {
    if log.traces.is_empty() {
        return Err(MutationError::InvalidValue(
            "Cannot sample from an empty event log".to_string(),
        ));
    }
    log.traces = (0..size)
        .map(|traceid| {
            let mut trace = log.traces.choose(rng).unwrap().clone();
            set_traceid(&mut trace, AttributeValue::String(traceid.to_string()));
            trace
        })
        .collect();

    Ok(())
}
