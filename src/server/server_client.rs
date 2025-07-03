use crate::server::events::{BufferId, EditorEvent};
use std::collections::HashSet;

pub struct Client {
    editor_events: Vec<EditorEvent>,
    subscribed_buffers: HashSet<BufferId>,
}

impl Client {
    pub(crate) fn new() -> Self {
        Self {
            editor_events: Vec::new(),
            subscribed_buffers: HashSet::new(),
        }
    }

    pub fn is_subscribed_to_buffer(&self, buffer_id: BufferId) -> bool {
        self.subscribed_buffers.contains(&buffer_id)
    }
    pub fn subscribe_to_buffer(&mut self, buffer_id: BufferId) {
        self.subscribed_buffers.insert(buffer_id);
    }

    pub fn unsubscribe_from_buffer(&mut self, buffer_id: BufferId) {
        self.subscribed_buffers.remove(&buffer_id);
    }

    pub fn get_event_queue(&mut self) -> Vec<EditorEvent> {
        // Clear the vector and return the contents
        self.editor_events.drain(..).collect()
    }

    pub fn push_to_event_queue(&mut self, event: EditorEvent) {
        self.editor_events.push(event);
    }
}
