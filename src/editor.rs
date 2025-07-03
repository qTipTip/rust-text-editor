use crate::client::editor_client::{ClientError, EditorClient};
use crate::editor::EditorError::NoActiveBuffer;
use crate::server::events::{BufferId, EditMode};
use crate::server::server_client::Client;
use std::path::PathBuf;

#[derive(Debug)]
pub enum EditorError {
    ClientError(ClientError),
    IoError(std::io::Error),
    NoActiveBuffer,
    BufferNotFound,
}

impl From<ClientError> for EditorError {
    fn from(err: ClientError) -> Self {
        EditorError::ClientError(err)
    }
}

impl From<std::io::Error> for EditorError {
    fn from(err: std::io::Error) -> Self {
        EditorError::IoError(err)
    }
}

pub type EditorResult<T> = Result<T, EditorError>;

pub struct Editor {
    client: EditorClient,
    current_buffer_id: Option<BufferId>,
    current_file: Option<PathBuf>,
    is_modified: bool,
    status_message: String,
    viewport_size: usize,
    scroll_offset: usize,
}

impl Editor {
    // Creation methods
    pub async fn new() -> EditorResult<Self> {
        Ok(Self {
            client: EditorClient::new().await?,
            current_buffer_id: None,
            current_file: None,
            is_modified: false,
            status_message: "Hello from rust-text-editor".to_string(),
            viewport_size: 0,
            scroll_offset: 0,
        })
    }
    pub async fn with_content(content: String) -> EditorResult<Self> {
        let mut client = EditorClient::new().await?;
        let buffer_id = client.create_buffer(Some(content)).await?;

        Ok(Self {
            client: client,
            current_buffer_id: Some(buffer_id),
            current_file: None,
            is_modified: false,
            status_message: "Hello from rust-text-editor".to_string(),
            viewport_size: 0,
            scroll_offset: 0,
        })
    }
    pub async fn open_file(path: PathBuf) -> EditorResult<Self> {
        todo!()
    }

    // Buffer management
    pub fn current_buffer_id(&self) -> Option<BufferId> {
        self.current_buffer_id
    }
    pub fn buffer_count(&self) -> usize {
        self.client.buffer_count()
    }
    pub async fn create_new_buffer(&mut self, content: Option<String>) -> EditorResult<BufferId> {
        todo!()
    }
    pub async fn switch_to_buffer(&mut self, buffer_id: BufferId) -> EditorResult<()> {
        todo!()
    }
    pub async fn get_current_buffer_content(&self) -> EditorResult<String> {
        match self.current_buffer_id {
            None => Err(NoActiveBuffer),
            Some(buffer_id) => Ok(self.client.get_content(buffer_id).await?),
        }
    }

    // Text operations
    pub async fn insert_char_at_cursor(&mut self, ch: char) -> EditorResult<()> {
        todo!()
    }
    pub async fn delete_char_at_cursor(&mut self) -> EditorResult<()> {
        todo!()
    }
    pub async fn insert_text_at_cursor(&mut self, text: &str) -> EditorResult<()> {
        todo!()
    }

    // Cursor operations
    pub async fn get_cursor_position(&self) -> EditorResult<usize> {
        todo!()
    }
    pub async fn set_cursor_position(&mut self, position: usize) -> EditorResult<()> {
        todo!()
    }
    pub async fn get_cursor_display_position(&self) -> EditorResult<(usize, usize)> {
        todo!()
    }
    pub async fn move_cursor_left(&mut self) -> EditorResult<()> {
        todo!()
    }
    pub async fn move_cursor_right(&mut self) -> EditorResult<()> {
        todo!()
    }
    pub async fn move_cursor_up(&mut self) -> EditorResult<()> {
        todo!()
    }
    pub async fn move_cursor_down(&mut self) -> EditorResult<()> {
        todo!()
    }

    // Modal operations
    pub async fn get_current_mode(&self) -> EditorResult<EditMode> {
        todo!()
    }
    pub async fn enter_normal_mode(&mut self) -> EditorResult<()> {
        todo!()
    }
    pub async fn enter_insert_mode(&mut self) -> EditorResult<()> {
        todo!()
    }
    pub async fn enter_visual_mode(&mut self) -> EditorResult<()> {
        todo!()
    }
    pub async fn enter_command_mode(&mut self) -> EditorResult<()> {
        todo!()
    }

    // File operations
    pub fn current_file_path(&self) -> Option<&PathBuf> {
        todo!()
    }
    pub fn is_modified(&self) -> bool {
        todo!()
    }
    pub async fn save(&mut self) -> EditorResult<()> {
        todo!()
    }
    pub async fn save_as(&mut self, path: PathBuf) -> EditorResult<()> {
        todo!()
    }

    // Display operations
    pub fn get_status_message(&self) -> &str {
        todo!()
    }
    pub fn set_status_message(&mut self, message: String) {
        todo!()
    }
    pub fn get_viewport_size(&self) -> usize {
        todo!()
    }
    pub fn set_viewport_size(&mut self, size: usize) {
        todo!()
    }
    pub fn get_scroll_offset(&self) -> usize {
        todo!()
    }
    pub fn set_scroll_offset(&mut self, offset: usize) {
        todo!()
    }
}
