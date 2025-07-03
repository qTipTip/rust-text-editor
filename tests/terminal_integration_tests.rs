use rust_text_editor::editor::Editor;
use rust_text_editor::server::events::EditMode;

#[tokio::test]
async fn test_editor_run_preparation() {
    let mut editor = Editor::with_content("hello\nworld".to_string()).await.unwrap();

    // Test cursor position to display coordinate conversion
    editor.set_cursor_position(8).await.unwrap(); // Position in "world"
    let (row, col) = editor.get_cursor_display_position().await.unwrap();
    assert_eq!(row, 1); // Second line
    assert_eq!(col, 2); // Third character in line
}

#[tokio::test]
async fn test_editor_viewport_and_scrolling() {
    let content = (0..50).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n");
    let mut editor = Editor::with_content(content).await.unwrap();

    // Set viewport size
    editor.set_viewport_size(10);
    assert_eq!(editor.get_viewport_size(), 10);

    // Test scrolling logic
    editor.set_cursor_position(500).await.unwrap(); // Move to later in document
    editor.update_scroll_for_cursor().await.unwrap();

    // Scroll offset should be adjusted to show cursor
    let scroll = editor.get_scroll_offset();
    let (cursor_row, _) = editor.get_cursor_display_position().await.unwrap();

    // Cursor should be visible in viewport
    assert!(cursor_row >= scroll);
    assert!(cursor_row < scroll + editor.get_viewport_size());
}

#[tokio::test]
async fn test_editor_status_line_info() {
    let mut editor = Editor::with_content("hello\nworld".to_string()).await.unwrap();

    // Test status line components
    let mode = editor.get_current_mode().await.unwrap();
    assert_eq!(mode, EditMode::Normal);

    let (row, col) = editor.get_cursor_display_position().await.unwrap();
    let status_info = editor.get_status_line_info().await.unwrap();

    // Should contain mode, cursor position, and other info
    assert!(status_info.contains("NORMAL"));
    assert!(status_info.contains(&format!("{}:{}", row + 1, col + 1))); // 1-indexed for display
}

#[tokio::test]
async fn test_editor_render_lines() {
    let content = "line1\nline2\nline3\nline4\nline5".to_string();
    let mut editor = Editor::with_content(content).await.unwrap();

    editor.set_viewport_size(3);
    editor.set_scroll_offset(1); // Skip first line

    let visible_lines = editor.get_visible_lines().await.unwrap();

    assert_eq!(visible_lines.len(), 3);
    assert_eq!(visible_lines[0], "line2");
    assert_eq!(visible_lines[1], "line3");
    assert_eq!(visible_lines[2], "line4");
}

#[tokio::test]
async fn test_editor_cursor_in_viewport() {
    let mut editor = Editor::with_content("hello\nworld\ntest".to_string()).await.unwrap();

    editor.set_viewport_size(2); // Small viewport
    editor.set_cursor_position(12).await.unwrap(); // In "test" line (line 2)

    let (cursor_row, cursor_col) = editor.get_cursor_display_position().await.unwrap();
    assert_eq!(cursor_row, 2);

    // Update scroll to show cursor
    editor.update_scroll_for_cursor().await.unwrap();

    let scroll = editor.get_scroll_offset();
    let viewport_cursor_row = cursor_row - scroll;

    // Cursor should be visible in viewport
    assert!(viewport_cursor_row < editor.get_viewport_size());
}

#[tokio::test]
async fn test_editor_input_handling_normal_mode() {
    let mut editor = Editor::with_content("test".to_string()).await.unwrap();

    // Should start in normal mode
    let mode = editor.get_current_mode().await.unwrap();
    assert_eq!(mode, EditMode::Normal);

    // Test mode switching
    editor.handle_normal_mode_key('i').await.unwrap();
    let mode = editor.get_current_mode().await.unwrap();
    assert_eq!(mode, EditMode::Insert);

    // Test cursor movement in normal mode
    editor.enter_normal_mode().await.unwrap();
    editor.set_cursor_position(2).await.unwrap();

    editor.handle_normal_mode_key('l').await.unwrap(); // Move right
    let pos = editor.get_cursor_position().await.unwrap();
    assert_eq!(pos, 3);

    editor.handle_normal_mode_key('h').await.unwrap(); // Move left
    let pos = editor.get_cursor_position().await.unwrap();
    assert_eq!(pos, 2);
}

#[tokio::test]
async fn test_editor_input_handling_insert_mode() {
    let mut editor = Editor::with_content("test".to_string()).await.unwrap();

    editor.enter_insert_mode().await.unwrap();
    editor.set_cursor_position(2).await.unwrap(); // Between 'e' and 's'

    // Insert character
    editor.handle_insert_mode_char('X').await.unwrap();
    let content = editor.get_current_buffer_content().await.unwrap();
    assert_eq!(content, "teXst");

    // Test backspace
    editor.handle_insert_mode_backspace().await.unwrap();
    let content = editor.get_current_buffer_content().await.unwrap();
    assert_eq!(content, "test");

    // Test escape to normal mode
    editor.handle_insert_mode_escape().await.unwrap();
    let mode = editor.get_current_mode().await.unwrap();
    assert_eq!(mode, EditMode::Normal);
}

#[tokio::test]
async fn test_editor_modification_tracking() {
    let mut editor = Editor::with_content("original".to_string()).await.unwrap();

    assert!(!editor.is_modified());

    // Make a change
    editor.enter_insert_mode().await.unwrap();
    editor.handle_insert_mode_char('!').await.unwrap();

    assert!(editor.is_modified());

    // Save should clear modified flag
    // (This would normally save to a file, but for test we just clear the flag)
    editor.mark_as_saved();
    assert!(!editor.is_modified());
}