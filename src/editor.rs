use crate::client::editor_client::{ClientError, EditorClient};
use crate::editor::EditorError::NoActiveBuffer;
use crate::server::events::{BufferId, EditMode};
use crate::syntax::SyntaxHighlighter;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use crossterm::{cursor, event, execute, terminal};
use std::fs::read_to_string;
use std::io::{Write, stdout};
use std::path::PathBuf;
use crossterm::style::Print;
use ropey::Rope;
use tree_sitter::Tree;

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
    syntax_highlighter: SyntaxHighlighter,
    syntax_tree: Option<Tree>,
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
            syntax_highlighter: SyntaxHighlighter::new(),
            syntax_tree: None,
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
            syntax_highlighter: SyntaxHighlighter::new(),
            syntax_tree: None,
        })
    }
    pub async fn open_file(path: PathBuf) -> EditorResult<Self> {
        let content = read_to_string(&path)?;

        let mut new_editor = Self::with_content(content.clone()).await?;
        new_editor.status_message = format!("Opened file: {}", path.display());
        new_editor.current_file = Some(path.clone());

        // Set up syntax highlighting for this file
        if let Err(e) = new_editor.syntax_highlighter.set_language_from_path(&path) {
            eprintln!("Failed to set syntax language: {}", e);
        }
        
        // Parse the content for syntax highlighting
        new_editor.syntax_tree = new_editor.syntax_highlighter.parse(&content);

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
                self.update_syntax_tree().await?;
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
                self.update_syntax_tree().await?;
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
                self.update_syntax_tree().await?;
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
        Ok(format!(
            "Cursor: ({}:{}) | Mode: {:?} | Status: {}",
            row + 1,
            col + 1,
            current_mode,
            status
        ))
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
        let (cursor_row, cursor_col) = self.get_cursor_display_position().await?;
        let viewport_row = cursor_row.saturating_sub(self.scroll_offset);
        Ok((viewport_row, cursor_col))
    }

    async fn update_syntax_tree(&mut self) -> EditorResult<()> {
        if let Some(buffer_id) = self.current_buffer_id {
            let content = self.client.get_content(buffer_id).await?;
            self.syntax_tree = self.syntax_highlighter.parse(&content);
        }
        Ok(())
    }

    // Terminal integration
    pub async fn run(&mut self) -> EditorResult<()> {
        if self.current_buffer_id.is_none() {
            let buffer_id = self.create_new_buffer(None).await?;
            self.current_buffer_id = Some(buffer_id);
        }

        enable_raw_mode().map_err(|e| EditorError::IoError(e))?;
        execute!(stdout(), EnterAlternateScreen).map_err(|e| EditorError::IoError(e))?;

        let result = self.event_loop().await;

        disable_raw_mode().map_err(|e| EditorError::IoError(e))?;
        execute!(stdout(), LeaveAlternateScreen).map_err(|e| EditorError::IoError(e))?;

        result
    }

    async fn event_loop(&mut self) -> EditorResult<()> {
        let (_, term_height) = terminal::size().map_err(|e| EditorError::IoError(e))?;
        self.viewport_size = (term_height as usize).saturating_sub(2);

        loop {
            self.update_scroll_for_cursor().await?;
            self.render().await?;

            if let Event::Key(key_event) = event::read().map_err(|e| EditorError::IoError(e))? {
                if self.handle_key_event(key_event).await? {
                    break;
                }
            }
        }

        Ok(())
    }

    async fn render(&mut self) -> EditorResult<()> {
        let (_, term_height) = terminal::size().map_err(|e| EditorError::IoError(e))?;
        let content_height = (term_height as usize).saturating_sub(2);

        // clear screen, move cursor to top left
        execute!(stdout(), cursor::Hide).map_err(|e| EditorError::IoError(e))?;
        execute!(stdout(), cursor::MoveTo(0, 0)).map_err(|e| EditorError::IoError(e))?;

        // Render visible lines with syntax highlighting
        let visible_lines = self.get_visible_lines().await?;

        for display_row in 0..content_height {
            execute!(stdout(), cursor::MoveTo(0, display_row as u16)).map_err(|e| EditorError::IoError(e))?;
            execute!(stdout(), terminal::Clear(terminal::ClearType::CurrentLine)).map_err(|e| EditorError::IoError(e))?;

            if display_row < visible_lines.len() {
                // Render line with syntax highlighting
                self.render_highlighted_line(&visible_lines[display_row], self.scroll_offset + display_row).await?;
            } else {
                // Render empty line indicator
                execute!(stdout(), Print("~")).map_err(|e| EditorError::IoError(e))?;
            }
        }

        // Render status lines
        self.render_status_lines().await?;

        // Position cursor
        let (viewport_row, viewport_col) = self.get_cursor_viewport_position().await?;
        execute!(stdout(), cursor::MoveTo(viewport_col as u16, viewport_row as u16)).map_err(|e| EditorError::IoError(e))?;
        execute!(stdout(), cursor::Show).map_err(|e| EditorError::IoError(e))?;

        stdout().flush().map_err(|e| EditorError::IoError(e))?;
        Ok(())

    }

    async fn render_highlighted_line(&mut self, line: &str, line_idx: usize) -> EditorResult<()> {
        // If we don't have a syntax tree, just print the line as-is
        let Some(ref tree) = self.syntax_tree else {
            execute!(stdout(), Print(line)).map_err(|e| EditorError::IoError(e))?;
            return Ok(());
        };

        // Get the buffer content as a rope for syntax highlighting
        let buffer_id = self.current_buffer_id.ok_or(NoActiveBuffer)?;
        let content = self.client.get_content(buffer_id).await?;
        let rope = Rope::from_str(&content);

        // Get highlights for this line
        let highlights = self.syntax_highlighter.highlight_line(&rope, line_idx, tree);

        if highlights.is_empty() {
            // No highlights, just print the line
            execute!(stdout(), Print(line)).map_err(|e| EditorError::IoError(e))?;
        } else {
            // Apply highlights
            let mut last_end = 0;
            let line_chars: Vec<char> = line.chars().collect();
 
            for (start, end, highlight_type) in highlights {
                // Print text before highlight
                if start > last_end {
                    let text: String = line_chars[last_end..start].iter().collect();
                    execute!(stdout(), Print(text)).map_err(|e| EditorError::IoError(e))?;
                }

                // Print highlighted text
                if end <= line_chars.len() {
                    let text: String = line_chars[start..end].iter().collect();
                    let style = highlight_type.to_style();
                    execute!(stdout(), crossterm::style::PrintStyledContent(style.apply(text))).map_err(|e| EditorError::IoError(e))?;
                }

                last_end = end;
            }

            // Print remaining text after last highlight
            if last_end < line_chars.len() {
                let text: String = line_chars[last_end..].iter().collect();
                execute!(stdout(), Print(text)).map_err(|e| EditorError::IoError(e))?;
            }
        }

        Ok(())
    }

    async fn render_status_lines(&mut self) -> EditorResult<()> {
        let (_, term_height) = terminal::size().map_err(|e| EditorError::IoError(e))?;
        let (cursor_row, cursor_col) = self.get_cursor_display_position().await?;

        // First status line - mode and file info
        execute!(stdout(), cursor::MoveTo(0, term_height - 2)).map_err(|e| EditorError::IoError(e))?;
        execute!(stdout(), terminal::Clear(terminal::ClearType::CurrentLine)).map_err(|e| EditorError::IoError(e))?;

        let status_info = self.get_status_line_info().await?;
        let file_info = match &self.current_file {
            Some(path) => format!(" | {}{}", path.display(), if self.is_modified { " [+]" } else { "" }),
            None => format!(" | [No Name]{}", if self.is_modified { " [+]" } else { "" }),
        };

        execute!(stdout(), Print(format!("{}{}", status_info, file_info))).map_err(|e| EditorError::IoError(e))?;

        // Second status line - help and statistics
        execute!(stdout(), cursor::MoveTo(0, term_height - 1)).map_err(|e| EditorError::IoError(e))?;
        execute!(stdout(), terminal::Clear(terminal::ClearType::CurrentLine)).map_err(|e| EditorError::IoError(e))?;

        let help_text = match self.get_current_mode().await? {
            EditMode::Normal => "i=Insert, v=Visual, :=Command",
            EditMode::Insert => "ESC=Normal",
            EditMode::Visual => "ESC=Normal",
            EditMode::Command => "ESC=Normal, Enter=Execute",
        };

        execute!(stdout(), Print(format!("{}:{} | {}", cursor_row + 1, cursor_col + 1, help_text))).map_err(|e| EditorError::IoError(e))?;

        Ok(())
    }

    async fn handle_key_event(&mut self, key_event: KeyEvent) -> EditorResult<bool> {
        // Only handle press and repeat events
        match key_event.kind {
            KeyEventKind::Release => return Ok(false),
            KeyEventKind::Press | KeyEventKind::Repeat => {}
        }

        // Mode-specific key handling
        let current_mode = self.get_current_mode().await?;
        match current_mode {
            EditMode::Normal => self.handle_normal_mode_input(key_event).await?,
            EditMode::Insert => self.handle_insert_mode_input(key_event).await?,
            EditMode::Visual => self.handle_visual_mode_input(key_event).await?,
            EditMode::Command => {
                if self.handle_command_mode_input(key_event).await? {
                    return Ok(true); // Signal to exit the event loop
                }
            }
        }

        Ok(false)
    }

    async fn handle_normal_mode_input(&mut self, key_event: KeyEvent) -> EditorResult<()> {
        match key_event.code {

            KeyCode::Char(ch) => {
                match ch {
                    ':' => {
                        self.enter_command_mode().await?;
                        self.status_message = ":".to_string();
                    }
                    _ => {
                        self.handle_normal_mode_key(ch).await?;
                        // Clear status message after successful command
                        if !self.status_message.starts_with("File modified!") {
                            self.status_message = "".to_string();
                        }
                    }
                }
            }

            KeyCode::Left => self.move_cursor_left().await?,
            KeyCode::Right => self.move_cursor_right().await?,
            KeyCode::Up => self.move_cursor_up().await?,
            KeyCode::Down => self.move_cursor_down().await?,
            _ => {}
        }
        Ok(())
    }

    async fn handle_insert_mode_input(&mut self, key_event: KeyEvent) -> EditorResult<()> {
        match key_event.code {
            KeyCode::Esc => {
                self.handle_insert_mode_escape().await?;
                self.status_message = "".to_string();
            }
            KeyCode::Char(ch) => {
                self.handle_insert_mode_char(ch).await?;
            }
            KeyCode::Backspace => {
                self.handle_insert_mode_backspace().await?;
            }
            KeyCode::Enter => {
                self.handle_insert_mode_char('\n').await?;
            }
            KeyCode::Left => self.move_cursor_left().await?,
            KeyCode::Right => self.move_cursor_right().await?,
            KeyCode::Up => self.move_cursor_up().await?,
            KeyCode::Down => self.move_cursor_down().await?,
            _ => {}
        }
        Ok(())
    }

    async fn handle_visual_mode_input(&mut self, key_event: KeyEvent) -> EditorResult<()> {
        match key_event.code {
            KeyCode::Esc => {
                self.enter_normal_mode().await?;
                self.status_message = "".to_string();
            }
            KeyCode::Char('h') => self.move_cursor_left().await?,
            KeyCode::Char('j') => self.move_cursor_down().await?,
            KeyCode::Char('k') => self.move_cursor_up().await?,
            KeyCode::Char('l') => self.move_cursor_right().await?,
            KeyCode::Left => self.move_cursor_left().await?,
            KeyCode::Right => self.move_cursor_right().await?,
            KeyCode::Up => self.move_cursor_up().await?,
            KeyCode::Down => self.move_cursor_down().await?,
            _ => {
                // For now, just show that visual mode is not fully implemented
                self.status_message = "Visual mode - ESC to return to Normal".to_string();
            }
        }
        Ok(())
    }

    async fn handle_command_mode_input(&mut self, key_event: KeyEvent) -> EditorResult<bool> {
        match key_event.code {
            KeyCode::Esc => {
                self.enter_normal_mode().await?;
                self.status_message = "".to_string();
            }
            KeyCode::Enter => {
                // Execute command (basic implementation)
                if self.execute_command().await? {
                    return Ok(true); // Signal to exit the event loop
                }
                self.enter_normal_mode().await?;
            }
            KeyCode::Char(ch) => {
                // Add character to command
                self.status_message.push(ch);
            }
            KeyCode::Backspace => {
                // Remove last character from command
                if self.status_message.len() > 1 {
                    self.status_message.pop();
                }
            }
            _ => {}
        }
        Ok(false)
    }

    async fn execute_command(&mut self) -> EditorResult<bool> {
        let command = self.status_message.trim_start_matches(':').to_string();

        match command.as_str() {
            "q" => {
                if self.is_modified {
                    self.status_message = "File modified! Use :q! to quit without saving".to_string();
                } else {
                    return Ok(true); // Signal to exit the event loop
                }
            }
            "q!" => {
                return Ok(true); // Signal to exit the event loop
            }
            "w" => {
                if let Err(e) = self.save().await {
                    self.status_message = format!("Save failed: {:?}", e);
                } else {
                    self.status_message = "File saved".to_string();
                }
            }
            "wq" => {
                if let Err(e) = self.save().await {
                    self.status_message = format!("Save failed: {:?}", e);
                } else {
                    return Ok(true); // Signal to exit the event loop
                }
            }
            _ if command.starts_with("w ") => {
                let filename = command.strip_prefix("w ").unwrap().trim();
                let path = PathBuf::from(filename);
                if let Err(e) = self.save_as(path).await {
                    self.status_message = format!("Save failed: {:?}", e);
                } else {
                    self.status_message = format!("Saved as {}", filename);
                }
            }
            _ => {
                self.status_message = format!("Unknown command: {}", command);
            }
        }

        Ok(false)
    }
}

