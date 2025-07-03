use crate::client::editor_client::{ClientError, EditorClient};
use crate::editor::EditorError::{IoError, NoActiveBuffer};
use crate::server::events::{BufferId, EditMode};
use crate::server::server_client::Client;
use std::fs::read_to_string;
use std::io::{stdout, Write};
use std::path::PathBuf;
use crossterm::cursor::Hide;
use crossterm::execute;
use crossterm::style::Print;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ropey::str_utils::char_to_byte_idx;

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

    pub async fn get_visible_lines(&self) -> EditorResult<Vec<String>> {
        let buffer_id = self.current_buffer_id.ok_or(NoActiveBuffer)?;
        let content = self.client.get_content(buffer_id).await?;

        let lines: Vec<&str> = content.lines().collect();
        let visible_lines = lines
            .iter()
            .skip(self.scroll_offset)
            .take(self.viewport_size)
            .map(|s| s.to_string())
            .collect();

        Ok(visible_lines)

    }
    pub async fn get_status_line_info(&self) -> EditorResult<String> {
        let (row, col) = self.get_cursor_display_position().await.unwrap();
        let current_mode = match self.get_current_mode().await? {
            EditMode::Normal => "NORMAL",
            EditMode::Insert => "INSERT",
            EditMode::Visual => "VISUAL",
            EditMode::Command => "COMMAND",
        };
        let status = self.get_status_message();
        Ok(format!("Cursor: ({}:{}) | Mode: {:?} | Status: {}", row + 1, col + 1, current_mode, status ))
    }
    pub async fn update_scroll_for_cursor(&mut self) -> EditorResult<()> {
        // Make sure the scroll-offset is modified to accommodate the cursor display positon.


        let (cursor_row, _) = self.get_cursor_display_position().await?;

        if cursor_row >= self.scroll_offset + self.viewport_size {
            self.scroll_offset = cursor_row - self.viewport_size + 1;
        }

        if cursor_row < self.scroll_offset {
            self.scroll_offset = cursor_row;
        }

        Ok(())
    }

    // Input handling
    pub async fn handle_normal_mode_key(&mut self, key: char) -> EditorResult<()> {
        match key {
            'i' => self.enter_insert_mode().await?,
            'v' => self.enter_visual_mode().await?,
            'k' => self.move_cursor_up().await?,
            'l' => self.move_cursor_right().await?,
            'h' => self.move_cursor_left().await?,
            'j' => self.move_cursor_down().await?,
            _ => {}
        }

        Ok(())
    }
    pub async fn handle_insert_mode_char(&mut self, ch: char) -> EditorResult<()> {
        Ok(self.insert_char_at_cursor(ch).await?)
    }
    pub async fn handle_insert_mode_backspace(&mut self) -> EditorResult<()> {
        self.move_cursor_left().await?;
        Ok(self.delete_char_at_cursor().await?)
    }
    pub async fn handle_insert_mode_escape(&mut self) -> EditorResult<()> {
        Ok(self.enter_normal_mode().await?)
    }

    // Utility methods
    pub fn mark_as_saved(&mut self) {
        self.is_modified = false;
    }
    pub async fn get_cursor_viewport_position(&self) -> EditorResult<(usize, usize)> {
        todo!()
    }

    // Terminal integration
    pub async fn run(&mut self) -> EditorResult<()> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;

        // let result = self.event_loop();
        disable_raw_mode()?;
        execute!(stdout(), LeaveAlternateScreen)?;
        
        Ok(())
    }
}

