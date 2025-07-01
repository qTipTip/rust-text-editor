use fs::read_to_string;
use crate::text_buffer::TextBuffer;
use crate::syntax::{SyntaxHighlighter, HighlightType};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::style::{Print, ResetColor, SetStyle};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode};
use crossterm::{cursor, event, execute, terminal, terminal::enable_raw_mode};
use std::{fs, io};
use std::io::{Write, stdout};
use std::path::PathBuf;
use tree_sitter::Tree;

pub struct Editor {
    buffer: TextBuffer,
    current_file: Option<PathBuf>,
    viewport_size: usize,
    scroll_offset: usize,
    status_message: String,
    syntax_highlighter: SyntaxHighlighter,
    syntax_tree: Option<Tree>,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            buffer: TextBuffer::new(),
            current_file: None,
            viewport_size: 0,
            scroll_offset: 0,
            status_message: "Rust Text Editor".to_string(),
            syntax_highlighter: SyntaxHighlighter::new(),
            syntax_tree: None,
        }
    }

    pub fn with_content(content: String) -> Self {
        let mut editor = Self{
            buffer: TextBuffer::from_string(content),
            current_file: None,
            scroll_offset: 0,
            viewport_size: 0,
            status_message: "Rust Text Editor".to_string(),
            syntax_highlighter: SyntaxHighlighter::new(),
            syntax_tree: None,
        };
        editor.update_syntax_tree();
        editor
    }

    pub fn open_file(path: PathBuf) -> io::Result<Self> {
        let content = read_to_string(&path)?;
        let mut editor = Self {
            buffer: TextBuffer::from_string(content),
            current_file: Some(path.clone()),
            scroll_offset: 0,
            viewport_size: 0,
            status_message: format!("Opened: {}", path.display()),
            syntax_highlighter: SyntaxHighlighter::new(),
            syntax_tree: None,
        };

        if let Err(e) = editor.syntax_highlighter.set_language_from_path(&path) {
            editor.status_message = format!("Warning: Could not set syntax highlighting: {}", e)
        }
        editor.update_syntax_tree();
        Ok(editor)
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
                self.buffer.mark_buffer_as_saved();
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
        self.buffer.mark_buffer_as_saved();
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
                self.render_line_with_highlighting(buffer_line);
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
                if self.is_modified() {
                    self.status_message = "File modified, press ctrl-q to quit without saving".to_string();
                    return Ok(false);
                }
                return Ok(true);
            }
            KeyCode::Char('r') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                if self.is_modified() {
                    self.buffer.reset_buffer();
                    self.update_syntax_tree();
                }
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
                self.update_syntax_tree();
            }
            KeyCode::Backspace => {
                self.buffer.delete_char();
                self.update_syntax_tree();
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
                self.update_syntax_tree();
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

    fn is_modified(&self) -> bool {
        self.buffer.is_modified()
    }

    fn update_syntax_tree(&mut self) {
        let content = self.buffer.get_content();
        self.syntax_tree = self.syntax_highlighter.parse(&content);
    }

    fn render_line_with_highlighting(&self, line_idx: usize) -> io::Result<()> {
        let buffer_contents = self.buffer.get_content_rope();
        let line = buffer_contents.line(line_idx);
        let line_str = line.to_string();

        let highlights = if let Some(tree) = &self.syntax_tree {
            self.syntax_highlighter.highlight_line(buffer_contents, line_idx, tree)
        } else {
            Vec::new()
        };

        if highlights.is_empty() {
            execute!(stdout(), Print(&line_str))?;
            return Ok(());
        }

        let mut current_pos = 0;
        let line_chars: Vec<char> = line_str.chars().collect();

        for (start, end, highlight_type) in highlights {
            if current_pos < start {
                let before_text: String = line_chars[current_pos..start].iter().collect();
                execute!(stdout(), ResetColor, Print(before_text))?;
            }

            if start < end && end <= line_chars.len() {
                let highlighted_text: String = line_chars[start..end].iter().collect();
                let style = highlight_type.to_style();
                execute!(stdout(), SetStyle(style), Print(highlighted_text), ResetColor)?;
            }

            current_pos = end;
        }

        if current_pos < line_chars.len() {
            let remaining_text: String = line_chars[current_pos..].iter().collect();
            execute!(stdout(), Print(remaining_text))?;
        }

        Ok(())
    }
}
