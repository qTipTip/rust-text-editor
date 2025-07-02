use std::collections::HashSet;
use crate::server::events::{BufferId, EditorEvent};

pub struct Client {
    editor_events: Vec<EditorEvent>,
    subscribed_buffers: HashSet<BufferId>
}

impl Client {
    pub(crate) fn new() -> Self{
        Self {
            editor_events: Vec::new(),
            subscribed_buffers: HashSet::new()
        }
    }
}