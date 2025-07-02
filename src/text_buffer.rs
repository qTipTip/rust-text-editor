use ropey::Rope;
use std::cmp::min;
use std::hash::{DefaultHasher, Hash, Hasher};
use crate::server::events::EditMode;

pub struct TextBuffer {
    content: Rope,
    original_content: Rope,
    original_content_hash: u64,
    original_cursor_position: usize,
    cursor_position: usize,
    content_hash: u64,
    edit_mode: EditMode
}

impl TextBuffer {
    pub fn new() -> TextBuffer {
        let content = Rope::new();
        let content_hash = Self::hash_rope(&content);
        TextBuffer {
            content: content.clone(),
            original_content: content,
            original_content_hash: content_hash,
            original_cursor_position: 0,
            cursor_position: 0,
            content_hash,
            edit_mode: EditMode::Normal,
        }
    }

    pub fn from_string(string: String) -> TextBuffer {
        let content = Rope::from(string);
        let cursor_position = content.len_chars();
        let content_hash = Self::hash_rope(&content);
        TextBuffer {
            content: content.clone(),
            cursor_position,
            original_cursor_position: cursor_position,
            original_content: content,
            original_content_hash: content_hash,
            content_hash,
            edit_mode: EditMode::Normal,
        }
    }

    pub fn get_content(&self) -> String {
        self.content.to_string()
    }

    pub fn get_content_rope(&self) -> &Rope {
        &self.content
    }
    pub fn get_cursor_position(&self) -> usize {
        self.cursor_position
    }

    pub fn set_cursor_position(&mut self, position: usize) -> Result<(), String> {
        if position > self.content.len_chars() {
            Err("Attempting to set cursor position out of bounds".to_string())
        } else {
            self.cursor_position = position;
            Ok(())
        }
    }
    pub fn insert_char_at_position(&mut self, position: usize, ch: char) {
        self.content.insert_char(position, ch);

        // If we insert at cursor position or to the left, we move the cursor position to the right
        if position <= self.cursor_position {
            self.move_cursor_right();
        }
    }

    pub fn delete_char_at_position(&mut self, position: usize) -> Result<char, String> {
        if position >= self.content.len_chars() {
            return Err("Position out of bounds".to_string());
        }

        let deleted_char = self.content.char(position);
        self.content.remove(position..position + 1);

        if position < self.cursor_position {
            self.cursor_position -= 1;
        }

        Ok(deleted_char)
    }
    pub fn insert_char(&mut self, ch: char) {
        self.content.insert_char(self.cursor_position, ch);
        self.cursor_position += 1;
        self.update_content_hash();
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.content
                .remove(self.cursor_position..self.cursor_position + 1);
            self.update_content_hash();
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.content.len_chars() {
            self.cursor_position += 1;
        }
    }

    pub fn move_cursor_up(&mut self) {
        let line_idx = self.content.char_to_line(self.cursor_position);
        if line_idx > 0 {
            let current_line_start = self.content.line_to_char(line_idx);
            let col_in_line = self.cursor_position - current_line_start;

            let prev_line_start = self.content.line_to_char(line_idx - 1);
            let prev_line_len = self.content.line(line_idx - 1).len_chars();

            self.cursor_position = prev_line_start + min(col_in_line, prev_line_len);
        }
    }

    fn update_content_hash(&mut self) {
        self.content_hash = Self::hash_rope(&self.content);
    }

    pub fn move_cursor_down(&mut self) {
        let line_idx = self.content.char_to_line(self.cursor_position);
        if line_idx < self.content.len_lines() - 1 {
            let current_line_start = self.content.line_to_char(line_idx);
            let col_in_line = self.cursor_position - current_line_start;

            let prev_line_start = self.content.line_to_char(line_idx + 1);
            let prev_line_len = self.content.line(line_idx + 1).len_chars();

            self.cursor_position = prev_line_start + min(col_in_line, prev_line_len);
        }
    }

    pub fn get_cursor_display_position(&self) -> (usize, usize) {
        // Get the index of the line the cursor is on
        let row_idx = self.content.char_to_line(self.cursor_position);
        // Get the index of the first character of the line
        let col_start_idx = self.content.line_to_char(row_idx);
        // Get the index of the cursor relative to the current line by subtracting the index of the first character.
        let col_idx = self.cursor_position - col_start_idx;
        (row_idx, col_idx)
    }

    pub fn get_rope_statistics(&self) -> String {
        let chars = self.content.len_chars();
        let lines = self.content.len_lines();
        let bytes = self.content.len_bytes();
        let chunks = self.content.chunks().count();

        let avg_chunk_size = if chunks > 0 { bytes / chunks } else { 0 };

        let efficiency_ratio = bytes as f64 / avg_chunk_size as f64;

        format!(
            "C:{} B:{} L:{} Ch:{} AvgChunk:{} Eff:{:.2}",
            chars, bytes, lines, chunks, avg_chunk_size, efficiency_ratio
        )
    }

    pub fn is_modified(&self) -> bool {
        if self.content_hash != self.original_content_hash {
            true
        } else {
            false
        }
    }

    pub fn mark_buffer_as_saved(&mut self) {
        self.original_content = self.content.clone();
        self.content_hash = Self::hash_rope(&self.content);
        self.original_content_hash = Self::hash_rope(&self.content);
        self.original_cursor_position = self.cursor_position;
    }

    fn hash_rope(rope: &Rope) -> u64 {
        // Iteratively hashes the chunks in a rope.
        let mut hasher = DefaultHasher::new();
        for chunk in rope.chunks() {
            chunk.hash(&mut hasher);
        }
        hasher.finish()
    }

    pub fn reset_buffer(&mut self) {
        self.content = self.original_content.clone();
        self.cursor_position = self.original_cursor_position;
        self.content_hash = Self::hash_rope(&self.content);
    }

    pub fn get_edit_mode(&self) -> EditMode {
        self.edit_mode.clone()
    }

    pub fn set_edit_mode(&mut self, mode: EditMode) {
        self.edit_mode = mode;
    }

}

#[cfg(test)]
mod tests {
    use crate::text_buffer::TextBuffer;

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
    fn test_cursor_movement_up() {
        let mut buffer = TextBuffer::from_string("hello\nworld".to_string());
        buffer.cursor_position = 9; // After the r
        buffer.move_cursor_up();
        assert_eq!(buffer.get_cursor_position(), 3);

        buffer.move_cursor_up();
        assert_eq!(buffer.get_cursor_position(), 3);
    }

    #[test]
    fn test_cursor_movement_down() {
        let mut buffer = TextBuffer::from_string("hello\nworld".to_string());
        buffer.cursor_position = 3; // After the r
        buffer.move_cursor_down();
        assert_eq!(buffer.get_cursor_position(), 9);

        buffer.move_cursor_down();
        assert_eq!(buffer.get_cursor_position(), 9);
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
        assert_eq!(buffer.get_cursor_position(), 2);
    }

    #[test]
    fn test_cursor_display_position_single_line() {
        let buffer = TextBuffer::from_string("hello".to_string());
        let (row, col) = buffer.get_cursor_display_position();

        assert_eq!(row, 0);
        assert_eq!(col, 5);
    }

    #[test]
    fn test_cursor_display_position_empty_buffer() {
        let buffer = TextBuffer::new();
        let (row, col) = buffer.get_cursor_display_position();

        assert_eq!(row, 0);
        assert_eq!(col, 0);
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

    #[test]
    fn test_modified_hash() {
        let mut buffer = TextBuffer::from_string("hello".to_string());
        buffer.insert_char(' ');
        assert!(buffer.is_modified());
        buffer.delete_char();
        assert!(!buffer.is_modified());
    }

    #[test]
    fn test_modified_hash_with_save() {
        let mut buffer = TextBuffer::from_string("hello".to_string());
        buffer.insert_char(' ');
        assert!(buffer.is_modified());
        buffer.mark_buffer_as_saved();
        assert!(!buffer.is_modified());
    }

    #[test]
    fn test_long_modified_hash() {
        let mut buffer = TextBuffer::from_string("hello".to_string());
        for char in 'a'..'z' {
            buffer.insert_char(char);
            assert!(buffer.is_modified());
        }

        for char in 'a'..'z' {
            assert!(buffer.is_modified());
            buffer.delete_char();
        }

        assert!(!buffer.is_modified());
    }

    #[test]
    fn test_reset_buffer() {
        let mut buffer = TextBuffer::from_string("hello".to_string());
        for char in 'a'..'z' {
            buffer.insert_char(char);
            assert!(buffer.is_modified());
        }

        buffer.reset_buffer();

        assert!(!buffer.is_modified());
        assert_eq!(buffer.get_content(), "hello");
    }

    #[test]
    fn test_set_cursor_position_valid() {
        let mut buffer = TextBuffer::from_string("hello".to_string());

        // Should succeed for valid positions
        assert!(buffer.set_cursor_position(0).is_ok());
        assert_eq!(buffer.get_cursor_position(), 0);

        assert!(buffer.set_cursor_position(3).is_ok());
        assert_eq!(buffer.get_cursor_position(), 3);

        assert!(buffer.set_cursor_position(5).is_ok()); // End of string
        assert_eq!(buffer.get_cursor_position(), 5);
    }

    #[test]
    fn test_set_cursor_position_invalid() {
        let mut buffer = TextBuffer::from_string("hello".to_string());

        // Should fail for out of bounds
        assert!(buffer.set_cursor_position(6).is_err());

        // Cursor should remain unchanged after failed set
        let original_pos = buffer.get_cursor_position();
        let _ = buffer.set_cursor_position(10);
        assert_eq!(buffer.get_cursor_position(), original_pos);
    }

    #[test]
    fn test_insert_char_at_position_beginning() {
        let mut buffer = TextBuffer::from_string("hello".to_string());
        buffer.set_cursor_position(3).unwrap(); // Cursor at 'l'

        buffer.insert_char_at_position(0, 'X');

        assert_eq!(buffer.get_content(), "Xhello");
        assert_eq!(buffer.get_cursor_position(), 4); // Cursor moved right
    }

    #[test]
    fn test_insert_char_at_position_middle() {
        let mut buffer = TextBuffer::from_string("hello".to_string());
        buffer.set_cursor_position(1).unwrap(); // Cursor at 'e'

        buffer.insert_char_at_position(3, 'X'); // Insert after cursor

        assert_eq!(buffer.get_content(), "helXlo");
        assert_eq!(buffer.get_cursor_position(), 1); // Cursor unchanged
    }

    #[test]
    fn test_insert_char_at_position_before_cursor() {
        let mut buffer = TextBuffer::from_string("hello".to_string());
        buffer.set_cursor_position(3).unwrap(); // Cursor at first 'l'

        buffer.insert_char_at_position(2, 'X'); // Insert before cursor

        assert_eq!(buffer.get_content(), "heXllo");
        assert_eq!(buffer.get_cursor_position(), 4); // Cursor moved right
    }

    #[test]
    fn test_insert_char_at_position_at_cursor() {
        let mut buffer = TextBuffer::from_string("hello".to_string());
        buffer.set_cursor_position(2).unwrap(); // Cursor at first 'l'

        buffer.insert_char_at_position(2, 'X'); // Insert exactly at cursor

        assert_eq!(buffer.get_content(), "heXllo");
        // What should happen to cursor when inserting AT cursor position?
        // Your current logic: position <= cursor_position, so cursor moves right
        assert_eq!(buffer.get_cursor_position(), 3);
    }

    #[test]
    fn test_delete_char_at_position_before_cursor() {
        let mut buffer = TextBuffer::from_string("hello".to_string());
        buffer.set_cursor_position(3).unwrap(); // Cursor at first 'l'

        buffer.delete_char_at_position(1); // Delete 'e'

        assert_eq!(buffer.get_content(), "hllo");
        assert_eq!(buffer.get_cursor_position(), 2); // Cursor moved left
    }

    #[test]
    fn test_delete_char_at_position_after_cursor() {
        let mut buffer = TextBuffer::from_string("hello".to_string());
        buffer.set_cursor_position(1).unwrap(); // Cursor at 'e'

        buffer.delete_char_at_position(3); // Delete first 'l'

        assert_eq!(buffer.get_content(), "helo");
        assert_eq!(buffer.get_cursor_position(), 1); // Cursor unchanged
    }

    #[test]
    fn test_delete_char_at_position_at_cursor() {
        let mut buffer = TextBuffer::from_string("hello".to_string());
        buffer.set_cursor_position(2).unwrap(); // Cursor at first 'l'

        buffer.delete_char_at_position(2); // Delete character AT cursor

        assert_eq!(buffer.get_content(), "helo");
        // What should cursor position be after deleting AT cursor?
        // Your current logic: position < cursor_position (false), so no change
        assert_eq!(buffer.get_cursor_position(), 2);
    }


    #[test]
    fn test_delete_char_out_of_bounds() {
        let mut buffer = TextBuffer::from_string("hi".to_string());

        // Should this panic, or gracefully handle?
        buffer.delete_char_at_position(5);
    }

    #[test]
    fn test_position_methods_with_unicode() {
        let mut buffer = TextBuffer::from_string("🦀é".to_string());
        buffer.set_cursor_position(1).unwrap(); // After crab emoji

        buffer.insert_char_at_position(0, 'A');
        assert_eq!(buffer.get_content(), "A🦀é");
        assert_eq!(buffer.get_cursor_position(), 2); // Cursor moved right

        buffer.delete_char_at_position(0); // Delete 'A'
        assert_eq!(buffer.get_content(), "🦀é");
        assert_eq!(buffer.get_cursor_position(), 1); // Cursor moved left
    }
}
