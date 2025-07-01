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
}

impl Editor {
    pub fn new() -> Self {
        Self {
            buffer: TextBuffer::new(),
            current_file: None,
            is_modified: false,
        }
    }

    pub fn with_content(content: String) -> Self {
        Self {
            buffer: TextBuffer::from_string(content),
            current_file: None,
            is_modified: false,
        }
    }

    pub fn open_file(path: PathBuf) -> io::Result<Self> {
        let content = read_to_string(&path)?;
        Ok(Self {
            buffer: TextBuffer::from_string(content),
            current_file: Some(path.clone()),
            is_modified: false,
        })
    }

    pub fn save_file(&mut self) -> io::Result<()> {
        match &self.current_file {
            None => {
                println!("Press ctrl-A to save-as");
                Ok(())
            }
            Some(path) => {
                let content = self.buffer.get_content();
                fs::write(path, content)?;
                self.is_modified = false;
                Ok(())
            }
        }
    }

    pub fn save_file_as(&mut self) -> io::Result<()> {
        let path = PathBuf::from("test_write.txt");
        self.current_file = Some(path.clone());
        let content = self.buffer.get_content();
        fs::write(path, content)?;
        self.is_modified = false;
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
        loop {
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
        // Flush the terminal, and set the cursor to (0,0)
        execute!(stdout(), terminal::Clear(terminal::ClearType::All))?;
        execute!(stdout(), cursor::MoveTo(0, 0))?;

        let buffer_contents = self.buffer.get_content_rope();
        // We then write contents to the screen.
        for (line_idx, line) in buffer_contents.lines().enumerate() {
            execute!(stdout(), cursor::MoveTo(0, line_idx as u16))?;
            execute!(stdout(), Print(line))?
        }

        // We compute the cursor-display position, and save it.
        let (row, col) = self.buffer.get_cursor_display_position();
        execute!(stdout(), cursor::MoveTo(col as u16, row as u16))?;
        execute!(stdout(), cursor::SavePosition)?;

        // Then we write the statusline
        let (_term_width, term_height) = terminal::size()?;
        execute!(stdout(), cursor::MoveTo(0, term_height - 1))?;
        execute!(
            stdout(),
            Print(format!(
                "Cursor: ({}, {}) Rope: ({})| Ctrl+Q to quit",
                row,
                col,
                self.buffer.get_rope_statistics()
            ))
        )?;

        // Finally, we move the cursor back to the display-position.
        execute!(stdout(), cursor::RestorePosition)?;
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> io::Result<bool> {
        match key_event.kind {
            KeyEventKind::Release => return Ok(false),
            KeyEventKind::Press | KeyEventKind::Repeat => {}
        }

        match key_event.code {
            KeyCode::Char('q') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                return Ok(true);
            }
            KeyCode::Char('s') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                match self.save_file() {
                    Ok(_) => {}
                    Err(_) => {}
                }
            }
            KeyCode::Char('a') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                match self.save_file_as() {
                    Ok(_) | Err(_) => {},
                }
            }
            KeyCode::Char(ch) => {
                self.buffer.insert_char(ch);
            }
            KeyCode::Backspace => {
                self.buffer.delete_char();
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
            }
            _ => {}
        }
        Ok(false)
    }
}
