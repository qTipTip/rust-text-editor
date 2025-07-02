use crate::server::client::Client;
use crate::server::events::{BufferId, ClientId, EditMode, EditorEvent, ServerError, ServerResult};
use crate::text_buffer::TextBuffer;
use std::collections::HashMap;
use crate::server::events::ServerError::BufferNotFound;

pub struct EditorServer {
    clients: HashMap<ClientId, Client>,
    buffers: HashMap<BufferId, TextBuffer>,
    buffer_owners: HashMap<BufferId, ClientId>, // One client owner per buffer
}

impl EditorServer {
    pub async fn new() -> Self {
        Self {
            clients: HashMap::new(),
            buffers: HashMap::new(),
            buffer_owners: HashMap::new(),
        }
    }
    pub async fn set_edit_mode(&mut self, buffer_id: BufferId, mode: EditMode) -> ServerResult<()> {
        match self.buffers.get_mut(&buffer_id) {
            Some(buffer) => {
                Ok(buffer.set_edit_mode(mode))
            }
            _ => Err(BufferNotFound)
        }
    }

    pub async fn get_edit_mode(&self, buffer_id: BufferId) -> ServerResult<EditMode> {
        match self.buffers.get(&buffer_id) {
            Some(buffer) => {
                Ok(buffer.get_edit_mode())
            }
            _ => Err(BufferNotFound)
        }
    }

    pub async fn get_client_events(&self, client_id: ClientId) -> ServerResult<Vec<EditorEvent>> {
        todo!()
    }

    pub async fn subscribe_to_buffer(
        &self,
        client_id: ClientId,
        buffer_id: BufferId,
    ) -> ServerResult<()> {
        todo!()
    }

    pub async fn move_cursor_left(&mut self, buffer_id: BufferId) -> ServerResult<()> {
        match self.buffers.get_mut(&buffer_id) {
            None => Err(BufferNotFound),
            Some(buffer) => {
                buffer.move_cursor_left();
                Ok(())
            }
        }
    }
    pub async fn move_cursor_right(&mut self, buffer_id: BufferId) -> ServerResult<()> {
        match self.buffers.get_mut(&buffer_id) {
            None => Err(BufferNotFound),
            Some(buffer) => {
                buffer.move_cursor_right();
                Ok(())
            }
        }    }
    pub async fn get_cursor_position(&self, buffer_id: BufferId) -> ServerResult<usize> {
        match self.buffers.get(&buffer_id) {
            None => Err(BufferNotFound),
            Some(buffer) => {
                Ok(buffer.get_cursor_position())
            }
        }
    }

    pub async fn set_cursor_position(
        &mut self,
        buffer_id: BufferId,
        position: i32,
    ) -> ServerResult<()> {
        match self.buffers.get_mut(&buffer_id) {
            None => Err(BufferNotFound),
            Some(buffer) => {
                buffer.set_cursor_position(position as usize);
                Ok(())
            }
        }
    }

    pub async fn delete_char(&mut self, buffer_id: BufferId, position: i32) -> ServerResult<()> {
        match self.buffers.get_mut(&buffer_id) {
            None => Err(ServerError::BufferNotFound),
            Some(buffer) => {
                buffer.delete_char_at_position(position as usize);
                Ok(())
            }
        }
    }
    pub async fn insert_char(
        &mut self,
        buffer_id: BufferId,
        position: i32,
        ch: char,
    ) -> ServerResult<()> {
        match self.buffers.get_mut(&buffer_id) {
            None => Err(ServerError::BufferNotFound),
            Some(buffer) => {
                buffer.insert_char_at_position(position as usize, ch);
                Ok(())
            }
        }
    }

    pub async fn get_buffer_content(&self, buffer_id: BufferId) -> ServerResult<String> {
        match self.buffers.get(&buffer_id) {
            None => Err(ServerError::BufferNotFound),
            Some(buffer) => Ok(buffer.get_content()),
        }
    }
    pub fn buffer_count(&self) -> usize {
        self.buffers.len()
    }
    pub async fn create_buffer(
        &mut self,
        client_id: ClientId,
        content: Option<String>,
    ) -> ServerResult<BufferId> {
        let client = match self.clients.get(&client_id) {
            None => {
                return Err(BufferNotFound)

            },
            Some(client) => client
        };

        let buffer = match content {
            None => TextBuffer::new(),
            Some(content) => TextBuffer::from_string(content),
        };

        let buffer_id = BufferId::new();
        self.buffers.insert(buffer_id, buffer);
        self.buffer_owners.insert(buffer_id, client_id);

        Ok(buffer_id)
    }

    pub fn is_client_connected(&self, client_id: ClientId) -> bool {
        self.clients.contains_key(&client_id)
    }

    pub async fn disconnect_client(&mut self, client_id: ClientId) -> ServerResult<()> {
        match self.clients.remove(&client_id) {
            None => Err(ServerError::ClientNotFound),
            Some(_) => {
                let buffers_owned_by_client: Vec<BufferId> = self.buffer_owners
                    .iter()
                    .filter(|(_, owner) | **owner == client_id)
                    .map(|(&buffer_id, _)|buffer_id)
                    .collect();

                for buffer_id in buffers_owned_by_client {
                    self.buffers.remove(&buffer_id);
                    self.buffer_owners.remove(&buffer_id);
                }

                Ok(())
            },
        }
    }

    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    pub async fn connect_client(&mut self) -> ServerResult<ClientId> {
        let client_id = ClientId::new();
        let client = Client::new();

        self.clients.insert(client_id, client);
        Ok(client_id)
    }
}
