use process_mining::{
    event_log::{Event, Trace},
    EventLog,
};

use crate::parsing::dir_name_trait::DirName;

pub trait EventMutator {
    /// Apply the mutation to a given event.
    fn apply_mut(&mut self, evt: &mut Event);

    fn apply(&mut self, evt: &Event) -> Event {
        let mut new_event = evt.clone();
        self.apply_mut(&mut new_event);
        new_event
    }
}

pub trait TraceMutator {
    /// Apply the mutation to a given trace.
    fn apply_mut(&mut self, trace: &mut Trace);

    fn apply(&mut self, trace: &Trace) -> Trace {
        let mut new_trace = trace.clone();
        self.apply_mut(&mut new_trace);
        new_trace
    }
}

pub trait LogMutator {
    /// Apply the mutation to an entire Event Log.
    fn apply_mut(&mut self, log: &mut EventLog);

    fn apply(&mut self, log: &EventLog) -> EventLog {
        let mut new_log = log.clone();
        self.apply_mut(&mut new_log);
        new_log
    }
}

impl<T> TraceMutator for T
where
    T: EventMutator,
{
    fn apply_mut(&mut self, trace: &mut Trace) {
        trace
            .events
            .iter_mut()
            .for_each(|event| self.apply_mut(event));
    }
}

impl<T> LogMutator for T
where
    T: TraceMutator,
{
    fn apply_mut(&mut self, log: &mut EventLog) {
        log.traces
            .iter_mut()
            .for_each(|trace| self.apply_mut(trace));
    }
}

pub trait LogMutatorWithAsDirName: LogMutator + DirName {}
impl<T: LogMutator + DirName> LogMutatorWithAsDirName for T {}

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
    fn apply_mut(&mut self, log: &mut EventLog) {
        self.mutations.iter_mut().for_each(|mutation| {
            mutation.apply_mut(log);
        });
    }
}
