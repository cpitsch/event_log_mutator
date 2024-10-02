use process_mining::{
    event_log::{Event, Trace},
    EventLog,
};
use thiserror::Error;

use crate::{
    parsing::traits::DirName,
    utils::{attributes::MissingAttributeError, io::IoError},
};

#[derive(Error, Debug)]
pub enum MutationError {
    // #[error("The {}-level attribute {} isn't present. Required by mutator {}", .1.level, .1.key, .0)]
    #[error("[{0}] Missing the {}-level attribute \"{}\".", .1.level, .1.key)]
    MissingAttributeError(&'static str, MissingAttributeError),
    #[error(transparent)]
    IoError(#[from] IoError),
    #[error("Invalid Value: {0}")]
    InvalidValue(&'static str),
}

pub type MutationResult<T> = Result<T, MutationError>;

pub trait EventMutator {
    /// Apply the mutation to a given event.
    fn apply_mut(&mut self, evt: &mut Event) -> MutationResult<()>;

    fn apply(&mut self, evt: &Event) -> MutationResult<Event> {
        let mut new_event = evt.clone();
        self.apply_mut(&mut new_event)?;
        Ok(new_event)
    }
}

pub trait TraceMutator {
    /// Apply the mutation to a given trace.
    fn apply_mut(&mut self, trace: &mut Trace) -> MutationResult<()>;

    fn apply(&mut self, trace: &Trace) -> MutationResult<Trace> {
        let mut new_trace = trace.clone();
        self.apply_mut(&mut new_trace)?;
        Ok(new_trace)
    }
}

pub trait LogMutator {
    /// Apply the mutation to an entire Event Log.
    fn apply_mut(&mut self, log: &mut EventLog) -> MutationResult<()>;

    fn apply(&mut self, log: &EventLog) -> MutationResult<EventLog> {
        let mut new_log = log.clone();
        self.apply_mut(&mut new_log)?;
        Ok(new_log)
    }
}

impl<T> TraceMutator for T
where
    T: EventMutator,
{
    fn apply_mut(&mut self, trace: &mut Trace) -> MutationResult<()> {
        for event in trace.events.iter_mut() {
            self.apply_mut(event)?;
        }
        Ok(())
    }
}

impl<T> LogMutator for T
where
    T: TraceMutator,
{
    fn apply_mut(&mut self, log: &mut EventLog) -> MutationResult<()> {
        for trace in log.traces.iter_mut() {
            self.apply_mut(trace)?;
        }
        Ok(())
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
    fn apply_mut(&mut self, log: &mut EventLog) -> MutationResult<()> {
        for mutation in self.mutations.iter_mut() {
            mutation.apply_mut(log)?;
        }
        Ok(())
    }
}
