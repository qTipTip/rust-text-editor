use crate::server::events::{ClientId, EditMode, ServerResult};

pub struct EditorServer {
}

impl EditorServer {
    pub async fn set_edit_mode(&self, p0: _, p1: _) {
        todo!()
    }

    pub async fn get_edit_mode(&self, p0: _) -> EditMode {
        todo!()
    }

    pub async fn get_client_events(&self, p0: _) {
        todo!()
    }

    pub async fn subscribe_to_buffer(&self, p0: _, p1: _) {
        todo!()
    }

    pub async fn move_cursor_left(&self, p0: _) {
        todo!()
    }
    pub async fn move_cursor_right(&self, p0: _) {
        todo!()
    }
    pub async fn get_cursor_position(&self, p0: _) -> usize {
        todo!()
    }

    pub async fn set_cursor_position(&self, p0: _, p1: i32) {
        todo!()
    }

    pub async fn delete_char(&self, p0: _, p1: i32) {
        todo!()
    }
    pub async fn insert_char(&self, p0: _, p1: i32, p2: char) {
        todo!()
    }

    pub async fn get_buffer_content(&self, p0: _) {
        todo!()
    }
    pub fn buffer_count(&self) -> usize {
        todo!()
    }
    pub async fn create_buffer(&self, p0: _, p1: Option<_>) {
        todo!()
    }

    pub fn is_client_connected(&self, p0: _) -> bool {
        todo!()
    }

    pub async fn disconnect_client(&self, client_id: ClientId) {
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