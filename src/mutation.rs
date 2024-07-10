use process_mining::{
    event_log::{Event, Trace},
    EventLog,
};

use crate::parsing::as_dir_name::AsDirName;

pub trait EventMutator {
    /// Apply the mutation to a given event.
    fn apply(&self, evt: &Event) -> Event;
}

pub trait TraceMutator {
    /// Apply the mutation to a given trace.
    fn apply(&self, trace: &Trace) -> Trace;
}

pub trait LogMutator {
    /// Apply the mutation to an entire Event Log.
    fn apply(&self, log: &EventLog) -> EventLog;
}

impl<T> TraceMutator for T
where
    T: EventMutator,
{
    fn apply(&self, trace: &Trace) -> Trace {
        let mut new_trace = trace.clone();
        new_trace
            .events
            .iter_mut()
            .for_each(|event| *event = self.apply(event));
        new_trace
    }
}

impl<T> LogMutator for T
where
    T: TraceMutator,
{
    fn apply(&self, log: &EventLog) -> EventLog {
        let mut new_log = log.clone();
        new_log.traces.iter_mut().for_each(|trace| {
            *trace = self.apply(trace);
        });
        new_log
    }
}

pub trait LogMutatorWithAsDirName: LogMutator + AsDirName {}
impl<T: LogMutator + AsDirName> LogMutatorWithAsDirName for T {}

/// A Mutation pipeline to apply a number of mutations to an event log at once.
#[derive(Default)]
pub struct MutationChain {
    pub mutations: Vec<Box<dyn LogMutatorWithAsDirName>>,
}

impl MutationChain {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a mutation to the pipeline.
    pub fn with_mutation<T: LogMutatorWithAsDirName + 'static>(mut self, mutation: T) -> Self {
        self.mutations.push(Box::new(mutation));
        self
    }
}

impl LogMutator for MutationChain {
    fn apply(&self, log: &EventLog) -> EventLog {
        let mut new_log = log.clone();
        self.mutations.iter().for_each(|mutation| {
            new_log = mutation.apply(&new_log);
        });

        new_log
    }
}
