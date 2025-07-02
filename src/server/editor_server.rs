use std::collections::HashMap;
use crate::server::client::Client;
use crate::server::events::{BufferId, ClientId, EditMode, EditorEvent, ServerError, ServerResult};
use crate::text_buffer::TextBuffer;

pub struct EditorServer {
    clients: HashMap<ClientId, Client>,
    buffers: HashMap<BufferId, TextBuffer>,
}

impl EditorServer {
    pub async fn new() -> Self {
        Self { clients: HashMap::new(), buffers: HashMap::new() }
    }
    pub async fn set_edit_mode(&self, buffer_id: BufferId, mode: EditMode) -> ServerResult<()> {
        todo!()
    }

    pub async fn get_edit_mode(&self, buffer_id: BufferId) -> ServerResult<EditMode> {
        todo!()
    }

    pub async fn get_client_events(&self, client_id: ClientId) -> ServerResult<Vec<EditorEvent>> {
        todo!()
    }

    pub async fn subscribe_to_buffer(&self, client_id: ClientId, buffer_id: BufferId) -> ServerResult<()> {
        todo!()
    }

    pub async fn move_cursor_left(&self, buffer_id: BufferId) -> ServerResult<()> {
        todo!()
    }
    pub async fn move_cursor_right(&self, buffer_id: BufferId) -> ServerResult<()> {
        todo!()
    }
    pub async fn get_cursor_position(&self, buffer_id: BufferId) -> ServerResult<usize> {
        todo!()
    }

    pub async fn set_cursor_position(&self, buffer_id: BufferId, position: i32) -> ServerResult<()> {
        todo!()
    }

    pub async fn delete_char(&self, buffer_id: BufferId, position: i32) -> ServerResult<()> {
        todo!()
    }
    pub async fn insert_char(&self, buffer_id: BufferId, position: i32, ch: char) -> ServerResult<()> {
        todo!()
    }

    pub async fn get_buffer_content(&self, buffer_id: BufferId) -> ServerResult<String> {
        match self.buffers.get(&buffer_id) {
            None => {
                Err(ServerError::BufferNotFound)
            }
            Some(buffer) => {
                Ok(buffer.get_content())
            }
        }
    }
    pub fn buffer_count(&self) -> usize {
        self.buffers.len()
    }
    pub async fn create_buffer(&mut self, client_id: ClientId, content: Option<String>) -> ServerResult<BufferId> {
        let buffer = match content {
            None => {
                TextBuffer::new()
            }
            Some(content) => {
                TextBuffer::from_string(content)
            }
        };

        let buffer_id = BufferId::new();
        self.buffers.insert(buffer_id, buffer);

        Ok(buffer_id)

    }

    pub fn is_client_connected(&self, client_id: ClientId) -> bool {
        todo!()
    }

    pub async fn disconnect_client(&mut self, client_id: ClientId) -> ServerResult<()> {
        match self.clients.remove(&client_id) {
            None => {}
            Some(client) => {
                
            }
        }
    }

    pub fn client_count(&self) -> usize {
        todo!()
    }


    pub async fn connect_client(&mut self) -> ServerResult<ClientId> {
        let client_id = ClientId::new();
        let client = Client {};

        self.clients.insert(client_id, client);
        Ok(ClientId::new())
    }
}