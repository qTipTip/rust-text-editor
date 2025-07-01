struct TextBuffer {
    content: String,
    cursor_position: usize,
}

impl TextBuffer {
    pub fn new() -> TextBuffer {
        TextBuffer {
            content: String::new(),
            cursor_position: 0,
        }
    }

    pub fn from_string(content: String) -> TextBuffer {
        let cursor_position = content.len();
        TextBuffer {
            content,
            cursor_position,
        }
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }
    pub fn get_cursor_position(&self) -> usize {
        self.cursor_position
    }

    pub fn insert_char(&mut self, ch: char) {
        self.content.insert(self.cursor_position, ch);
        self.cursor_position += ch.len_utf8();
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            let prev_char_boundary = self.get_previous_character_boundary();
            self.content.remove(prev_char_boundary);
            self.cursor_position = prev_char_boundary;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position = self.get_previous_character_boundary();
        }
    }
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.content.len() {
            self.cursor_position = self.get_next_character_boundary();
        }
    }

    pub fn get_cursor_display_position(&self) -> (usize, usize) {
        let content_before_cursor = &self.content[..self.cursor_position];
        let rows = content_before_cursor.lines().count() - 1;
        let cols = content_before_cursor.lines().last().unwrap_or("").len();
        (rows, cols)
    }

    fn get_next_character_boundary(&self) -> usize {
        let mut pos = self.cursor_position;
        while pos < self.content.len() {
            pos += 1;
            if self.content.is_char_boundary(pos) {
                break;
            }
        }
        pos
    }

    fn get_previous_character_boundary(&self) -> usize {
        let mut pos = self.cursor_position;
        while pos > 0 {
            pos -= 1;
            if self.content.is_char_boundary(pos) {
                break
            }
        }
        pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer_is_empty() {
        let buffer = TextBuffer::new();
        assert_eq!(buffer.get_content(), "");
        assert_eq!(buffer.get_cursor_position(), 0);
    }

    #[test]
    fn test_insert_single_char() {
        let mut buffer = TextBuffer::new();
        buffer.insert_char('a');

        assert_eq!(buffer.get_content(), "a");
        assert_eq!(buffer.get_cursor_position(), 1);
    }

    #[test]
    fn test_insert_multiple_chars() {
        let mut buffer = TextBuffer::new();
        buffer.insert_char('h');
        buffer.insert_char('e');
        buffer.insert_char('l');
        buffer.insert_char('l');
        buffer.insert_char('o');

        assert_eq!(buffer.get_content(), "hello");
        assert_eq!(buffer.get_cursor_position(), 5);
    }

    #[test]
    fn test_delete_char() {
        let mut buffer = TextBuffer::from_string("hello".to_string());
        buffer.delete_char();

        assert_eq!(buffer.get_content(), "hell");
        assert_eq!(buffer.get_cursor_position(), 4);
    }

    #[test]
    fn test_delete_char_empty_buffer() {
        let mut buffer = TextBuffer::new();
        buffer.delete_char(); // Should not panic

        assert_eq!(buffer.get_content(), "");
        assert_eq!(buffer.get_cursor_position(), 0);
    }

    #[test]
    fn test_cursor_movement_left() {
        let mut buffer = TextBuffer::from_string("hello".to_string());
        buffer.move_cursor_left();

        assert_eq!(buffer.get_cursor_position(), 4);

        buffer.move_cursor_left();
        assert_eq!(buffer.get_cursor_position(), 3);
    }

    #[test]
    fn test_cursor_movement_right() {
        let mut buffer = TextBuffer::from_string("hello".to_string());
        buffer.cursor_position = 0; // Move to start

        buffer.move_cursor_right();
        assert_eq!(buffer.get_cursor_position(), 1);

        buffer.move_cursor_right();
        assert_eq!(buffer.get_cursor_position(), 2);
    }

    #[test]
    fn test_cursor_boundaries() {
        let mut buffer = TextBuffer::new();

        // Can't move left when at start
        buffer.move_cursor_left();
        assert_eq!(buffer.get_cursor_position(), 0);

        // Can't move right when at end
        buffer.move_cursor_right();
        assert_eq!(buffer.get_cursor_position(), 0);
    }

    #[test]
    fn test_unicode_support() {
        let mut buffer = TextBuffer::new();
        buffer.insert_char('🦀'); // Rust crab emoji
        buffer.insert_char('é'); // Accented character

        assert_eq!(buffer.get_content(), "🦀é");
        // Unicode characters take multiple bytes
        assert!(buffer.get_cursor_position() > 2);
    }

    #[test]
    fn test_cursor_display_position_single_line() {
        let buffer = TextBuffer::from_string("hello".to_string());
        let (row, col) = buffer.get_cursor_display_position();

        assert_eq!(row, 0);
        assert_eq!(col, 5);
    }

    #[test]
    fn test_cursor_display_position_multiple_lines() {
        let mut buffer = TextBuffer::from_string("hello\nworld".to_string());
        let (row, col) = buffer.get_cursor_display_position();

        assert_eq!(row, 1); // Second line
        assert_eq!(col, 5); // "world".len()
    }

    #[test]
    fn test_insert_at_middle() {
        let mut buffer = TextBuffer::from_string("hllo".to_string());
        buffer.cursor_position = 1; // Position between 'h' and 'l'
        buffer.insert_char('e');

        assert_eq!(buffer.get_content(), "hello");
        assert_eq!(buffer.get_cursor_position(), 2);
    }
}

fn main() {
    let mut buffer = TextBuffer::new();
    // buffer.insert_char('🦀'); // Rust crab emoji
    // buffer.insert_char('é'); // Accented character
    for c in { 'a'..'z' } {
        buffer.insert_char(c);
    }
    println!("{}", buffer.get_content());
}