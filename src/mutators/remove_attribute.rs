use crate::mutation::EventMutator;

/// A Mutation to remove an attribute key from all events.
pub struct RemoveAttributeMutation {
    /// The key to remove
    key: String,
}

impl RemoveAttributeMutation {
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into() }
    }
}

impl EventMutator for RemoveAttributeMutation {
    fn apply(&self, evt: &process_mining::event_log::Event) -> process_mining::event_log::Event {
        let mut new_event = evt.clone();
        new_event.attributes.retain(|attr| attr.key != self.key);
        new_event
    }
}
