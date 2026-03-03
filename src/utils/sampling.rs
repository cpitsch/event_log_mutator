use std::collections::HashSet;

use process_mining::core::event_data::case_centric::{AttributeValue, EventLog};
use rand::{rngs::StdRng, seq::SliceRandom};

use crate::{
    mutation::{MutationError, MutationResult},
    utils::attributes::{get_traceid, set_traceid},
};

use super::attributes::AttributeResult;

// TODO: Take log as first arg and maybe Option<&mut StdRng> instead?
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

    let mut new_log = log.clone_without_traces();
    new_log.traces.reserve_exact(size);

    // TODO: Sort the traces according to original order?
    // TODO: Use `rand::partial_shuffle` and `Vec::split_off`
    // But both of these wouldn't be backwards compatible
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

    // TODO: Why am I not just using indices? It wouldn't be semantically equivalent
    // if the log contains duplicate traceids, but which would be better?
    // Then again, if there were duplicate traceids, the sampled log could have
    // more than `size` traces, which is unexpected behavior.
    let retain_traceids: HashSet<_> = log
        .traces
        .choose_multiple(rng, size)
        .map(|trace| get_traceid(trace).cloned())
        .collect::<AttributeResult<_>>()
        .map_err(|e| MutationError::AttributeError("SampleWithoutReplacement", e))?;
    log.traces
        .retain(|trace| retain_traceids.contains(get_traceid(trace).unwrap()));
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

pub fn split_log(
    log: &EventLog,
    size: usize,
    rng: &mut StdRng,
) -> MutationResult<(EventLog, EventLog)> {
    if size > log.traces.len() {
        return Err(MutationError::InvalidValue(format!(
            "Cannot split an event log with a half larger than the whole ({}>{})",
            size,
            log.traces.len()
        )));
    }

    let mut log_1 = log.clone_without_traces();
    let mut log_2 = log.clone_without_traces();

    let mut all_traces = log.traces.clone();

    // If log_1 is larger than half, shuffling "for" log_2 is less work.
    if size > log.traces.len() / 2 {
        let complement_size = all_traces.len() - size;
        all_traces.partial_shuffle(rng, complement_size);
        log_2.traces = all_traces.split_off(complement_size);
        log_1.traces = all_traces
    } else {
        all_traces.partial_shuffle(rng, size);
        log_1.traces = all_traces.split_off(size);
        log_2.traces = all_traces;
    }

    Ok((log_1, log_2))
}
