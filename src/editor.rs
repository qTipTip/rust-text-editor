use fs::read_to_string;
use crate::text_buffer::TextBuffer;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::style::Print;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode};
use crossterm::{cursor, event, execute, terminal, terminal::enable_raw_mode};
use std::{fs, io};
use std::io::{Write, stdout};
use std::path::PathBuf;

pub struct Editor {
    buffer: TextBuffer,
    current_file: Option<PathBuf>,
    is_modified: bool,
    viewport_size: usize,
    scroll_offset: usize,
    status_message: String,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            buffer: TextBuffer::new(),
            current_file: None,
            viewport_size: 0,
            scroll_offset: 0,
            is_modified: false,
            status_message: "Rust Text Editor".to_string(),
        }
    }

    pub fn with_content(content: String) -> Self {
        Self {
            buffer: TextBuffer::from_string(content),
            current_file: None,
            scroll_offset: 0,
            viewport_size: 0,
            is_modified: false,
            status_message: "Rust Text Editor".to_string(),
        }
    }

    pub fn open_file(path: PathBuf) -> io::Result<Self> {
        let content = read_to_string(&path)?;
        Ok(Self {
            buffer: TextBuffer::from_string(content),
            current_file: Some(path.clone()),
            scroll_offset: 0,
            viewport_size: 0,
            is_modified: false,
            status_message: format!("Opened: {}", path.display()),
        })
    }

    pub fn save_file(&mut self) -> io::Result<()> {
        match &self.current_file {
            None => {
                self.status_message = "Unable to save. Press Ctrl-A to save as".to_string();
                Ok(())
            }
            Some(path) => {
                let content = self.buffer.get_content();
                fs::write(path, content)?;
                self.is_modified = false;
                self.status_message = format!("Saved: {}", path.display());
                Ok(())
            }
        }
    }

    pub fn save_file_as(&mut self) -> io::Result<()> {
        let path = PathBuf::from("test_write.txt");
        self.current_file = Some(path.clone());
        let content = self.buffer.get_content();
        fs::write(&path, content)?;
        self.is_modified = false;
        self.status_message = format!("Saved: {}", path.display());
        Ok(())
    }

    pub fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;

        let result = self.event_loop();

        disable_raw_mode()?;
        execute!(stdout(), LeaveAlternateScreen)?;

        result
    }

    fn event_loop(&mut self) -> io::Result<()> {

        let (_, term_height) = terminal::size()?;
        self.viewport_size = (term_height as usize).saturating_sub(2); // Reserve 2 lines for status

        loop {
            self.update_scroll();
            self.render()?;

            if let Event::Key(key_event) = event::read()? {
                if self.handle_key_event(key_event)? {
                    break;
                }
            }
        }
        Ok(())
    }

    fn mark_modified(&mut self) {
        if !self.is_modified {
            self.is_modified = true;
        }
    }

    fn render(&self) -> io::Result<()> {

        let (_, term_height) = terminal::size()?;
        let content_height = (term_height as usize).saturating_sub(2);
        
        
        // Flush the terminal, and set the cursor to (0,0)
        execute!(stdout(), cursor::Hide)?;
        // execute!(stdout(), terminal::Clear(terminal::ClearType::All))?;
        execute!(stdout(), cursor::MoveTo(0, 0))?;

        // Render only visible lines
        let buffer_contents = self.buffer.get_content_rope();
        let total_lines = buffer_contents.len_lines();
        // We then write contents to the screen.
        for display_row in 0..content_height {
            execute!(stdout(), cursor::MoveTo(0, display_row as u16))?;

            let buffer_line = self.scroll_offset + display_row;
            if buffer_line < total_lines {
                let line = buffer_contents.line(buffer_line);
                execute!(stdout(), terminal::Clear(terminal::ClearType::CurrentLine))?;
                execute!(stdout(), Print(line))?;
            } else {
                execute!(stdout(), terminal::Clear(terminal::ClearType::CurrentLine))?;
                execute!(stdout(), Print("~"))?;
            }
        }


        // Then we write the statusline
        let (row, col) = self.buffer.get_cursor_display_position();
        self.write_statusline(row, col)?;

        // Finally, we move the cursor back to the display-position clamped to the viewport.
        let viewport_row = row.saturating_sub(self.scroll_offset);
        execute!(stdout(), cursor::MoveTo(col as u16, viewport_row as u16))?;
        execute!(stdout(), cursor::Show)?;
        stdout().flush()?;
        Ok(())
    }

    fn write_statusline(&self, row: usize, col: usize) -> io::Result<()> {
        let (_term_width, term_height) = terminal::size()?;
        execute!(stdout(), cursor::MoveTo(0, term_height - 2))?;
        execute!(stdout(), terminal::Clear(terminal::ClearType::CurrentLine))?;
        execute!(stdout(), Print(&self.status_message))?;
        execute!(stdout(), cursor::MoveTo(0, term_height - 1))?;
        execute!(stdout(), terminal::Clear(terminal::ClearType::CurrentLine))?;
        execute!(
            stdout(),
            Print(format!(
                "Cursor: ({}, {}) Rope: ({})| Ctrl+Q to quit",
                row,
                col,
                self.buffer.get_rope_statistics()
            ))
        )?;
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> io::Result<bool> {
        match key_event.kind {
            KeyEventKind::Release => return Ok(false),
            KeyEventKind::Press | KeyEventKind::Repeat => {}
        }

        match key_event.code {
            KeyCode::Char('q') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                if self.is_modified {
                    self.status_message = "File modified, press ctrl-q to quit without saving".to_string();
                    return Ok(false);
                }
                return Ok(true);
            }
            KeyCode::Char('s') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                match self.save_file() {
                    Ok(_) => {}
                    Err(_) => {
                        self.status_message = "Failed to save file".to_string();
                    }
                }
            }
            KeyCode::Char('a') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                match self.save_file_as() {
                    Ok(_) | Err(_) => {},
                }
            }
            KeyCode::Char(ch) => {
                self.buffer.insert_char(ch);
                self.mark_modified();
            }
            KeyCode::Backspace => {
                self.buffer.delete_char();
                self.mark_modified();
            }
            KeyCode::Left => {
                self.buffer.move_cursor_left();
            }
            KeyCode::Right => {
                self.buffer.move_cursor_right();
            }
            KeyCode::Up => {
                self.buffer.move_cursor_up();
            }
            KeyCode::Down => {
                self.buffer.move_cursor_down();
            }
            KeyCode::Enter => {
                self.buffer.insert_char('\n');
                self.mark_modified();
            }
            _ => {}
        }
        Ok(false)
    }

    fn update_scroll(&mut self) {

        let (cursor_row, _) = self.buffer.get_cursor_display_position();

        if cursor_row >= self.scroll_offset + self.viewport_size {
            self.scroll_offset = cursor_row - self.viewport_size + 1;
        }

        if cursor_row < self.scroll_offset {
            self.scroll_offset = cursor_row;
        }
    }
}
