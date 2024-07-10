use crate::{mutation::EventMutator, parsing::dir_name_trait::DirName};

/// A Mutation to remove an attribute key from all events.
#[derive(DirName)]
pub struct AttributeRemover {
    /// The key to remove
    #[dirname(rename = "")]
    key: String,
}

impl AttributeRemover {
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into() }
    }
}

impl EventMutator for AttributeRemover {
    fn apply(&self, evt: &process_mining::event_log::Event) -> process_mining::event_log::Event {
        let mut new_event = evt.clone();
        new_event.attributes.retain(|attr| attr.key != self.key);
        new_event
    }
}
