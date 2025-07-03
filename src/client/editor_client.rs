use crate::server::client::Client;
use crate::server::editor_server::EditorServer;
use crate::server::events::{BufferId, ClientId, EditMode, ServerError};

pub struct EditorClient {
    server: EditorServer,
    client_id: ClientId,
    active_buffers: Vec<BufferId>,
}

#[derive(Debug)]
pub enum ClientError {
    ServerError(ServerError),
    BufferNotFound,
}

impl From<ServerError> for ClientError {
    fn from(err: ServerError) -> ClientError {
        ClientError::ServerError(err)
    }
}

pub type ClientResult<T> = Result<T, ClientError>;

impl EditorClient {
    pub async fn new() -> ClientResult<Self> {
        // Your implementation here
        todo!()
    }

    pub async fn create_buffer(&mut self, content: Option<String>) -> ClientResult<BufferId> {
        // Your implementation here
        todo!()
    }

    pub async fn get_content(&self, buffer_id: BufferId) -> ClientResult<String> {
        // Your implementation here
        todo!()
    }

    pub async fn insert_char(&mut self, buffer_id: BufferId, position: usize, ch: char) -> ClientResult<()> {
        // Your implementation here
        todo!()
    }

    pub async fn delete_char(&mut self, buffer_id: BufferId, position: usize) -> ClientResult<()> {
        // Your implementation here
        todo!()
    }

    pub async fn get_cursor_position(&self, buffer_id: BufferId) -> ClientResult<usize> {
        // Your implementation here
        todo!()
    }

    pub async fn set_cursor_position(&mut self, buffer_id: BufferId, position: usize) -> ClientResult<()> {
        // Your implementation here
        todo!()
    }

    pub async fn move_cursor_right(&mut self, buffer_id: BufferId) -> ClientResult<()> {
        // Your implementation here
        todo!()
    }

    pub async fn get_mode(&self, buffer_id: BufferId) -> ClientResult<EditMode> {
        // Your implementation here
        todo!()
    }

    pub async fn set_mode(&mut self, buffer_id: BufferId, mode: EditMode) -> ClientResult<()> {
        // Your implementation here
        todo!()
    }

    pub fn buffer_count(&self) -> usize {
        // Your implementation here
        todo!()
    }
}
