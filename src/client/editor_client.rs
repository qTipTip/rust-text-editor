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
        let mut server = EditorServer::new().await;
        let client_id = server.connect_client().await?;

        Ok(Self {
            server,
            client_id,
            active_buffers: Vec::new(),
        })
    }

    pub async fn create_buffer(&mut self, content: Option<String>) -> ClientResult<BufferId> {
        let buffer_id = self.server.create_buffer(self.client_id, content).await?;
        self.active_buffers.push(buffer_id);
        Ok(buffer_id)
    }

    pub async fn get_content(&self, buffer_id: BufferId) -> ClientResult<String> {
        Ok(self.server.get_buffer_content(buffer_id).await?)
    }

    pub async fn insert_char(
        &mut self,
        buffer_id: BufferId,
        position: usize,
        ch: char,
    ) -> ClientResult<()> {
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

    pub async fn set_cursor_position(
        &mut self,
        buffer_id: BufferId,
        position: usize,
    ) -> ClientResult<()> {
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
        self.active_buffers.len()
    }
}
