use crate::server::events::{BufferId, ClientId, EditMode, EditorEvent, ServerResult};

pub struct EditorServer {
}

impl EditorServer {
    pub async fn set_edit_mode(&self, buffer_id: BufferId, mode: EditMode) -> ServerResult<()> {
        todo!()
    }

    pub async fn get_edit_mode(&self, buffer_id: BufferId) -> ServerResult<EditMode> {
        todo!()
    }

    pub async fn get_client_events(&self, client_id: ClientId) -> ServerResult<Vec<EditorEvent>> {
        todo!()
    }

    pub async fn subscribe_to_buffer(&self, client_id: ClientId, buffer_id: BufferId) -> ServerResult<()>{
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
        todo!()
    }
    pub fn buffer_count(&self) -> usize {
        todo!()
    }
    pub async fn create_buffer(&self, client_id: ClientId, content: Option<String>) -> ServerResult<BufferId> {
        todo!()
    }

    pub fn is_client_connected(&self, client_id: ClientId) -> bool {
        todo!()
    }

    pub async fn disconnect_client(&self, client_id: ClientId) -> ServerResult<()>{
        todo!()
    }

    pub fn client_count(&self) -> usize {
        todo!()
    }

    pub async fn new() -> Self {
        Self {}
    }
    pub async fn connect_client(&self) -> ServerResult<ClientId> {
        todo!()
    }
}