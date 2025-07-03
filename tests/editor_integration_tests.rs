use rust_text_editor::editor::Editor;
use std::path::PathBuf;
use std::fs;
use tempfile::TempDir;
use rust_text_editor::server::events::EditMode;

#[tokio::test]
async fn test_editor_creation() {
    let editor = Editor::new().await.unwrap();

    // Should start with no active buffer
    assert!(editor.current_buffer_id().is_none());
    assert_eq!(editor.buffer_count(), 0);
}

#[tokio::test]
async fn test_editor_with_content() {
    let content = "Hello, world!";
    let editor = Editor::with_content(content.to_string()).await.unwrap();

    // Should have one buffer with the content
    assert!(editor.current_buffer_id().is_some());
    assert_eq!(editor.buffer_count(), 1);

    let buffer_content = editor.get_current_buffer_content().await.unwrap();
    assert_eq!(buffer_content, content);
}

#[tokio::test]
async fn test_editor_open_file() {
    // Create a temporary file
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    let content = "File content\nSecond line";
    fs::write(&file_path, content).unwrap();

    let editor = Editor::open_file(file_path.clone()).await.unwrap();

    // Should have the file content loaded
    assert!(editor.current_buffer_id().is_some());
    assert_eq!(editor.current_file_path(), Some(&file_path));

    let buffer_content = editor.get_current_buffer_content().await.unwrap();
    assert_eq!(buffer_content, content);
}

#[tokio::test]
async fn test_editor_text_operations() {
    let mut editor = Editor::with_content("abc".to_string()).await.unwrap();

    // Insert character at position
    editor.insert_char_at_cursor('X').await.unwrap();
    let content = editor.get_current_buffer_content().await.unwrap();
    assert_eq!(content, "abcX"); // Should insert at end (cursor starts at end)

    // Move cursor and insert
    editor.set_cursor_position(1).await.unwrap();
    editor.insert_char_at_cursor('Y').await.unwrap();
    let content = editor.get_current_buffer_content().await.unwrap();
    assert_eq!(content, "aYbcX");
}

#[tokio::test]
async fn test_editor_cursor_operations() {
    let mut editor = Editor::with_content("hello\nworld".to_string()).await.unwrap();

    // Test cursor positioning
    editor.set_cursor_position(3).await.unwrap();
    let pos = editor.get_cursor_position().await.unwrap();
    assert_eq!(pos, 3);

    // Test cursor movement
    editor.move_cursor_right().await.unwrap();
    let pos = editor.get_cursor_position().await.unwrap();
    assert_eq!(pos, 4);

    editor.move_cursor_left().await.unwrap();
    let pos = editor.get_cursor_position().await.unwrap();
    assert_eq!(pos, 3);

    // Test cursor display position (row, col)
    let (row, col) = editor.get_cursor_display_position().await.unwrap();
    assert_eq!(row, 0); // First line
    assert_eq!(col, 3); // Third column
}

#[tokio::test]
async fn test_editor_multi_line_cursor() {
    let mut editor = Editor::with_content("hello\nworld\ntest".to_string()).await.unwrap();

    // Set cursor to second line, third character
    editor.set_cursor_position(8).await.unwrap(); // h-e-l-l-o-\n-w-o
    let (row, col) = editor.get_cursor_display_position().await.unwrap();
    assert_eq!(row, 1); // Second line (0-indexed)
    assert_eq!(col, 2); // Third character in line

    // Test vertical movement
    editor.move_cursor_down().await.unwrap();
    let (row, col) = editor.get_cursor_display_position().await.unwrap();
    assert_eq!(row, 2); // Third line
    assert_eq!(col, 2); // Same column

    editor.move_cursor_up().await.unwrap();
    let (row, col) = editor.get_cursor_display_position().await.unwrap();
    assert_eq!(row, 1); // Back to second line
    assert_eq!(col, 2); // Same column
}

#[tokio::test]
async fn test_editor_modal_operations() {
    let mut editor = Editor::with_content("test".to_string()).await.unwrap();

    // Should start in Normal mode
    let mode = editor.get_current_mode().await.unwrap();
    assert_eq!(mode, EditMode::Normal);

    // Switch to Insert mode
    editor.enter_insert_mode().await.unwrap();
    let mode = editor.get_current_mode().await.unwrap();
    assert_eq!(mode, EditMode::Insert);

    // Switch back to Normal mode
    editor.enter_normal_mode().await.unwrap();
    let mode = editor.get_current_mode().await.unwrap();
    assert_eq!(mode, EditMode::Normal);

    // Test other modes
    editor.enter_visual_mode().await.unwrap();
    let mode = editor.get_current_mode().await.unwrap();
    assert_eq!(mode, EditMode::Visual);
}

#[tokio::test]
async fn test_editor_file_operations() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("save_test.txt");

    let mut editor = Editor::with_content("Original content".to_string()).await.unwrap();

    // Save to new file
    editor.save_as(file_path.clone()).await.unwrap();
    assert_eq!(editor.current_file_path(), Some(&file_path));
    assert!(!editor.is_modified());

    // Verify file was saved
    let saved_content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(saved_content, "Original content");

    // Modify content
    editor.insert_char_at_cursor('!').await.unwrap();
    assert!(editor.is_modified());

    // Save again (should update existing file)
    editor.save().await.unwrap();
    assert!(!editor.is_modified());

    let updated_content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(updated_content, "Original content!");
}

#[tokio::test]
async fn test_editor_buffer_management() {
    let mut editor = Editor::new().await.unwrap();

    // Create first buffer
    let buffer1_id = editor.create_new_buffer(Some("Buffer 1".to_string())).await.unwrap();
    assert_eq!(editor.buffer_count(), 1);
    assert_eq!(editor.current_buffer_id(), Some(buffer1_id));

    // Create second buffer
    let buffer2_id = editor.create_new_buffer(Some("Buffer 2".to_string())).await.unwrap();
    assert_eq!(editor.buffer_count(), 2);
    assert_eq!(editor.current_buffer_id(), Some(buffer2_id)); // Should switch to new buffer

    // Switch between buffers
    editor.switch_to_buffer(buffer1_id).await.unwrap();
    assert_eq!(editor.current_buffer_id(), Some(buffer1_id));
    let content = editor.get_current_buffer_content().await.unwrap();
    assert_eq!(content, "Buffer 1");

    editor.switch_to_buffer(buffer2_id).await.unwrap();
    assert_eq!(editor.current_buffer_id(), Some(buffer2_id));
    let content = editor.get_current_buffer_content().await.unwrap();
    assert_eq!(content, "Buffer 2");
}

#[tokio::test]
async fn test_editor_error_handling() {
    let mut editor = Editor::new().await.unwrap();

    // Operations on no buffer should fail gracefully
    assert!(editor.get_current_buffer_content().await.is_err());
    assert!(editor.insert_char_at_cursor('x').await.is_err());
    assert!(editor.get_cursor_position().await.is_err());

    // Invalid file operations
    let invalid_path = PathBuf::from("/nonexistent/directory/file.txt");
    assert!(Editor::open_file(invalid_path).await.is_err());
}

#[tokio::test]
async fn test_editor_display_info() {
    let mut editor = Editor::with_content("hello\nworld".to_string()).await.unwrap();

    // Test status message
    let status = editor.get_status_message();
    assert!(!status.is_empty());

    editor.set_status_message("Custom status".to_string());
    assert_eq!(editor.get_status_message(), "Custom status");

    // Test viewport and scrolling info
    editor.set_viewport_size(20);
    assert_eq!(editor.get_viewport_size(), 20);

    let scroll_offset = editor.get_scroll_offset();
    assert_eq!(scroll_offset, 0); // Should start at top
}