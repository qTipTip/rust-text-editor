use crate::client::editor_client::{ClientError, EditorClient};
use crate::editor::EditorError::{IoError, NoActiveBuffer};
use crate::server::events::{BufferId, EditMode};
use crate::server::server_client::Client;
use std::fs::read_to_string;
use std::path::PathBuf;

#[derive(Debug)]
pub enum EditorError {
    ClientError(ClientError),
    IoError(std::io::Error),
    NoActiveBuffer,
    BufferNotFound,
    NoFilePath,
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
        let content = read_to_string(&path)?;

        let mut new_editor = Self::with_content(content).await?;
        new_editor.status_message = format!("Opened file: {}", path.display());
        new_editor.current_file = Some(path);

        Ok(new_editor)
    }

    // Buffer management
    pub fn current_buffer_id(&self) -> Option<BufferId> {
        self.current_buffer_id
    }
    pub fn buffer_count(&self) -> usize {
        self.client.buffer_count()
    }
    pub async fn create_new_buffer(&mut self, content: Option<String>) -> EditorResult<BufferId> {
        let buffer_id = self.client.create_buffer(content).await?;
        self.current_buffer_id = Some(buffer_id);
        Ok(buffer_id)
    }
    pub async fn switch_to_buffer(&mut self, buffer_id: BufferId) -> EditorResult<()> {
        self.current_buffer_id = Some(buffer_id);
        Ok(())
    }
    pub async fn get_current_buffer_content(&self) -> EditorResult<String> {
        match self.current_buffer_id {
            None => Err(NoActiveBuffer),
            Some(buffer_id) => Ok(self.client.get_content(buffer_id).await?),
        }
    }

    // Text operations
    pub async fn insert_char_at_cursor(&mut self, ch: char) -> EditorResult<()> {
        match self.current_buffer_id {
            None => Err(NoActiveBuffer),
            Some(buffer_id) => {
                self.client
                    .insert_char(
                        buffer_id,
                        self.client.get_cursor_position(buffer_id).await?,
                        ch,
                    )
                    .await?;
                self.is_modified = true;
                Ok(())
            }
        }
    }
    pub async fn delete_char_at_cursor(&mut self) -> EditorResult<()> {
        match self.current_buffer_id {
            None => Err(NoActiveBuffer),
            Some(buffer_id) => {
                self.client
                    .delete_char(buffer_id, self.client.get_cursor_position(buffer_id).await?)
                    .await?;
                self.is_modified = true;
                Ok(())
            }
        }
    }
    pub async fn insert_text_at_cursor(&mut self, text: &str) -> EditorResult<()> {
        match self.current_buffer_id {
            None => Err(NoActiveBuffer),
            Some(buffer_id) => {
                for ch in text.chars() {
                    self.client
                        .insert_char(
                            buffer_id,
                            self.client.get_cursor_position(buffer_id).await?,
                            ch,
                        )
                        .await?;
                }
                self.is_modified = true;
                Ok(())
            }
        }
    }

    // Cursor operations
    pub async fn get_cursor_position(&self) -> EditorResult<usize> {
        match self.current_buffer_id {
            None => Err(NoActiveBuffer),
            Some(buffer_id) => Ok(self.client.get_cursor_position(buffer_id).await?),
        }
    }
    pub async fn set_cursor_position(&mut self, position: usize) -> EditorResult<()> {
        match self.current_buffer_id {
            None => Err(NoActiveBuffer),
            Some(buffer_id) => Ok(self.client.set_cursor_position(buffer_id, position).await?),
        }
    }
    pub async fn get_cursor_display_position(&self) -> EditorResult<(usize, usize)> {
        match self.current_buffer_id {
            None => Err(NoActiveBuffer),
            Some(buffer_id) => Ok(self.client.get_cursor_display_position(buffer_id).await?),
        }
    }
    pub async fn move_cursor_left(&mut self) -> EditorResult<()> {
        match self.current_buffer_id {
            None => Err(NoActiveBuffer),
            Some(buffer_id) => Ok(self.client.move_cursor_left(buffer_id).await?),
        }
    }
    pub async fn move_cursor_right(&mut self) -> EditorResult<()> {
        match self.current_buffer_id {
            None => Err(NoActiveBuffer),
            Some(buffer_id) => Ok(self.client.move_cursor_right(buffer_id).await?),
        }
    }
    pub async fn move_cursor_up(&mut self) -> EditorResult<()> {
        match self.current_buffer_id {
            None => Err(NoActiveBuffer),
            Some(buffer_id) => Ok(self.client.move_cursor_up(buffer_id).await?),
        }
    }
    pub async fn move_cursor_down(&mut self) -> EditorResult<()> {
        match self.current_buffer_id {
            None => Err(NoActiveBuffer),
            Some(buffer_id) => Ok(self.client.move_cursor_down(buffer_id).await?),
        }
    }

    // Modal operations
    pub async fn get_current_mode(&self) -> EditorResult<EditMode> {
        match self.current_buffer_id {
            None => Err(NoActiveBuffer),
            Some(buffer_id) => Ok(self.client.get_mode(buffer_id).await?),
        }
    }
    pub async fn enter_normal_mode(&mut self) -> EditorResult<()> {
        match self.current_buffer_id {
            None => Err(NoActiveBuffer),
            Some(buffer_id) => Ok(self.client.set_mode(buffer_id, EditMode::Normal).await?),
        }
    }
    pub async fn enter_insert_mode(&mut self) -> EditorResult<()> {
        match self.current_buffer_id {
            None => Err(NoActiveBuffer),
            Some(buffer_id) => Ok(self.client.set_mode(buffer_id, EditMode::Insert).await?),
        }
    }
    pub async fn enter_visual_mode(&mut self) -> EditorResult<()> {
        match self.current_buffer_id {
            None => Err(NoActiveBuffer),
            Some(buffer_id) => Ok(self.client.set_mode(buffer_id, EditMode::Visual).await?),
        }
    }
    pub async fn enter_command_mode(&mut self) -> EditorResult<()> {
        match self.current_buffer_id {
            None => Err(NoActiveBuffer),
            Some(buffer_id) => Ok(self.client.set_mode(buffer_id, EditMode::Command).await?),
        }
    }

    // File operations
    pub fn current_file_path(&self) -> Option<&PathBuf> {
        self.current_file.as_ref()
    }
    pub fn is_modified(&self) -> bool {
        self.is_modified
    }
    pub async fn save(&mut self) -> EditorResult<()> {
        match (&self.current_file, self.current_buffer_id) {
            (Some(file_path), Some(buffer_id)) => {
                let content = self.client.get_content(buffer_id).await?;
                std::fs::write(file_path, content)?;
                self.is_modified = false;
                self.status_message = format!("Saved: {}", file_path.display());
                Ok(())
            }
            (None, Some(_)) => {
                Err(EditorError::NoFilePath) // Or create a new error variant like NoFilePath
            }
            (_, None) => Err(EditorError::NoActiveBuffer),
        }
    }
    pub async fn save_as(&mut self, path: PathBuf) -> EditorResult<()> {
        match self.current_buffer_id {
            Some(buffer_id) => {
                let content = self.client.get_content(buffer_id).await?;
                std::fs::write(&path, content)?;
                self.current_file = Some(path.clone());
                self.is_modified = false;
                self.status_message = format!("Saved as: {}", path.display());
                Ok(())
            }
            None => Err(EditorError::NoActiveBuffer),
        }
    }

    // Display operations
    pub fn get_status_message(&self) -> &str {
        self.status_message.as_str()
    }
    pub fn set_status_message(&mut self, message: String) {
        self.status_message = message;
    }
    pub fn get_viewport_size(&self) -> usize {
        self.viewport_size
    }
    pub fn set_viewport_size(&mut self, size: usize) {
        self.viewport_size = size;
    }
    pub fn get_scroll_offset(&self) -> usize {
        self.scroll_offset
    }
    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset;
    }
}
