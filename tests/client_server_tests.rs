use rust_text_editor::server::editor_server::EditorServer;
use tokio::time::{timeout, Duration};
use std::collections::HashMap;
use rust_text_editor::server::events::{BufferId, ClientId, EditMode, EditorEvent};

#[tokio::test]
async fn test_server_creation() {
    let server = EditorServer::new().await;
    assert_eq!(server.buffer_count(), 0);
    assert_eq!(server.client_count(), 0);
}

#[tokio::test]
async fn test_client_connection() {
    let mut server = EditorServer::new().await;
    let client_id = server.connect_client().await.unwrap();

    assert_eq!(server.client_count(), 1);
    assert!(server.is_client_connected(client_id));
}

#[tokio::test]
async fn test_client_disconnection() {
    let mut server = EditorServer::new().await;
    let client_id = server.connect_client().await.unwrap();

    server.disconnect_client(client_id).await.unwrap();
    assert_eq!(server.client_count(), 0);
    assert!(!server.is_client_connected(client_id));
}

#[tokio::test]
async fn test_buffer_creation() {
    let mut server = EditorServer::new().await;
    let client_id = server.connect_client().await.unwrap();

    let buffer_id = server.create_buffer(client_id, None).await.unwrap();
    assert_eq!(server.buffer_count(), 1);

    let content = server.get_buffer_content(buffer_id).await.unwrap();
    assert_eq!(content, "");
}

#[tokio::test]
async fn test_buffer_creation_with_content() {
    let mut server = EditorServer::new().await;
    let client_id = server.connect_client().await.unwrap();

    let initial_content = "Hello, World!";
    let buffer_id = server.create_buffer(client_id, Some(initial_content.to_string())).await.unwrap();

    let content = server.get_buffer_content(buffer_id).await.unwrap();
    assert_eq!(content, initial_content);
}

#[tokio::test]
async fn test_buffer_edit_operations() {
    let mut server = EditorServer::new().await;
    let client_id = server.connect_client().await.unwrap();
    let buffer_id = server.create_buffer(client_id, Some("abc".to_string())).await.unwrap();

    // Insert character at position
    server.insert_char(buffer_id, 1, 'X').await.unwrap();
    let content = server.get_buffer_content(buffer_id).await.unwrap();
    assert_eq!(content, "aXbc");

    // Delete character
    server.delete_char(buffer_id, 1).await.unwrap();
    let content = server.get_buffer_content(buffer_id).await.unwrap();
    assert_eq!(content, "abc");
}

#[tokio::test]
async fn test_cursor_operations() {
    let mut server = EditorServer::new().await;
    let client_id = server.connect_client().await.unwrap();
    let buffer_id = server.create_buffer(client_id, Some("hello\nworld".to_string())).await.unwrap();

    // Set cursor position
    server.set_cursor_position(buffer_id, 3).await.unwrap();
    let pos = server.get_cursor_position(buffer_id).await.unwrap();
    assert_eq!(pos, 3);

    // Move cursor operations
    server.move_cursor_right(buffer_id).await.unwrap();
    let pos = server.get_cursor_position(buffer_id).await.unwrap();
    assert_eq!(pos, 4);

    server.move_cursor_left(buffer_id).await.unwrap();
    let pos = server.get_cursor_position(buffer_id).await.unwrap();
    assert_eq!(pos, 3);
}

// Add these tests to your existing tests/client_server_tests.rs file
// Or create a new test file: tests/server_cursor_tests.rs

use rust_text_editor::server::*;

#[tokio::test]
async fn test_cursor_movement_up() {
    let mut server = EditorServer::new().await;
    let client_id = server.connect_client().await.unwrap();

    let buffer_id = server.create_buffer(client_id, Some("hello\nworld\ntest".to_string())).await.unwrap();

    let initial_pos = server.get_cursor_position(buffer_id).await.unwrap();
    assert_eq!(initial_pos, 16);

    server.move_cursor_up(buffer_id).await.unwrap();
    let pos_after_up1 = server.get_cursor_position(buffer_id).await.unwrap();
    assert_eq!(pos_after_up1, 10);

    server.move_cursor_up(buffer_id).await.unwrap();
    let pos_after_up2 = server.get_cursor_position(buffer_id).await.unwrap();
    assert_eq!(pos_after_up2, 4);

    server.move_cursor_up(buffer_id).await.unwrap();
    let pos_after_up3 = server.get_cursor_position(buffer_id).await.unwrap();
    assert_eq!(pos_after_up3, 4);
}

#[tokio::test]
async fn test_cursor_movement_down() {
    let mut server = EditorServer::new().await;
    let client_id = server.connect_client().await.unwrap();

    // Create buffer with multi-line content and set cursor to beginning
    let buffer_id = server.create_buffer(client_id, Some("hello\nworld\ntest".to_string())).await.unwrap();
    server.set_cursor_position(buffer_id, 0).await.unwrap(); // Start at beginning

    // Move cursor down once (should go to start of "world" line, position 6)
    server.move_cursor_down(buffer_id).await.unwrap();
    let pos_after_down1 = server.get_cursor_position(buffer_id).await.unwrap();
    assert_eq!(pos_after_down1, 6); // Start of "world" line

    // Move cursor down again (should go to start of "test" line, position 12)
    server.move_cursor_down(buffer_id).await.unwrap();
    let pos_after_down2 = server.get_cursor_position(buffer_id).await.unwrap();
    assert_eq!(pos_after_down2, 12); // Start of "test" line

    // Move cursor down again (should stay at bottom)
    server.move_cursor_down(buffer_id).await.unwrap();
    let pos_after_down3 = server.get_cursor_position(buffer_id).await.unwrap();
    assert_eq!(pos_after_down3, 12); // Should not move past bottom
}

#[tokio::test]
async fn test_cursor_movement_up_down_column_preservation() {
    let mut server = EditorServer::new().await;
    let client_id = server.connect_client().await.unwrap();
    
    let buffer_id = server.create_buffer(client_id, Some("hello world\nhi\nhello again".to_string())).await.unwrap();

    // Set cursor to position 8 (in "hello world" line, after "hello wo")
    server.set_cursor_position(buffer_id, 8).await.unwrap();

    // Move down to shorter line "hi" - cursor should go to end of "hi" (position 14)
    server.move_cursor_down(buffer_id).await.unwrap();
    let pos_after_down = server.get_cursor_position(buffer_id).await.unwrap();
    assert_eq!(pos_after_down, 14); // End of "hi"

    // Move down again, should end up after he in hello.
    server.move_cursor_down(buffer_id).await.unwrap();
    let pos_after_down2 = server.get_cursor_position(buffer_id).await.unwrap();
    assert_eq!(pos_after_down2, 17);

    server.move_cursor_up(buffer_id).await.unwrap();
    let pos_after_up = server.get_cursor_position(buffer_id).await.unwrap();
    assert_eq!(pos_after_up, 14);
    
    server.move_cursor_up(buffer_id).await.unwrap();
    let pos_after_up2 = server.get_cursor_position(buffer_id).await.unwrap();
    assert_eq!(pos_after_up2, 2);
}

#[tokio::test]
async fn test_cursor_movement_single_line() {
    let mut server = EditorServer::new().await;
    let client_id = server.connect_client().await.unwrap();

    // Create buffer with single line
    let buffer_id = server.create_buffer(client_id, Some("single line".to_string())).await.unwrap();

    // Set cursor to middle
    server.set_cursor_position(buffer_id, 5).await.unwrap();
    let initial_pos = server.get_cursor_position(buffer_id).await.unwrap();

    // Moving up/down on single line should not change position
    server.move_cursor_up(buffer_id).await.unwrap();
    let pos_after_up = server.get_cursor_position(buffer_id).await.unwrap();
    assert_eq!(pos_after_up, initial_pos);

    server.move_cursor_down(buffer_id).await.unwrap();
    let pos_after_down = server.get_cursor_position(buffer_id).await.unwrap();
    assert_eq!(pos_after_down, initial_pos);
}

#[tokio::test]
async fn test_cursor_movement_empty_buffer() {
    let mut server = EditorServer::new().await;
    let client_id = server.connect_client().await.unwrap();

    // Create empty buffer
    let buffer_id = server.create_buffer(client_id, None).await.unwrap();

    // Cursor should start at 0
    let initial_pos = server.get_cursor_position(buffer_id).await.unwrap();
    assert_eq!(initial_pos, 0);

    // Moving up/down on empty buffer should not change position
    server.move_cursor_up(buffer_id).await.unwrap();
    let pos_after_up = server.get_cursor_position(buffer_id).await.unwrap();
    assert_eq!(pos_after_up, 0);

    server.move_cursor_down(buffer_id).await.unwrap();
    let pos_after_down = server.get_cursor_position(buffer_id).await.unwrap();
    assert_eq!(pos_after_down, 0);
}

#[tokio::test]
async fn test_cursor_movement_error_handling() {
    let mut server = EditorServer::new().await;
    let client_id = server.connect_client().await.unwrap();

    // Try to move cursor on non-existent buffer
    let fake_buffer_id = BufferId::new();
    assert!(server.move_cursor_up(fake_buffer_id).await.is_err());
    assert!(server.move_cursor_down(fake_buffer_id).await.is_err());
}

#[tokio::test]
async fn test_all_cursor_movements_together() {
    let mut server = EditorServer::new().await;
    let client_id = server.connect_client().await.unwrap();

    // Create a 3x3 grid of text
    let buffer_id = server.create_buffer(client_id, Some("abc\ndef\nghi".to_string())).await.unwrap();

    // Start at beginning (position 0, should be 'a')
    server.set_cursor_position(buffer_id, 0).await.unwrap();

    // Move right twice to 'c'
    server.move_cursor_right(buffer_id).await.unwrap();
    server.move_cursor_right(buffer_id).await.unwrap();
    let pos = server.get_cursor_position(buffer_id).await.unwrap();
    assert_eq!(pos, 2); // At 'c'

    // Move down to 'f'
    server.move_cursor_down(buffer_id).await.unwrap();
    let pos = server.get_cursor_position(buffer_id).await.unwrap();
    assert_eq!(pos, 6); // At 'f'

    // Move down to 'i'
    server.move_cursor_down(buffer_id).await.unwrap();
    let pos = server.get_cursor_position(buffer_id).await.unwrap();
    assert_eq!(pos, 10); // At 'i'

    // Move left to 'h'
    server.move_cursor_left(buffer_id).await.unwrap();
    let pos = server.get_cursor_position(buffer_id).await.unwrap();
    assert_eq!(pos, 9); // At 'h'

    // Move up to 'e'
    server.move_cursor_up(buffer_id).await.unwrap();
    let pos = server.get_cursor_position(buffer_id).await.unwrap();
    assert_eq!(pos, 5); // At 'e'

    // Move up to 'b'
    server.move_cursor_up(buffer_id).await.unwrap();
    let pos = server.get_cursor_position(buffer_id).await.unwrap();
    assert_eq!(pos, 1); // At 'b'
}

#[tokio::test]
async fn test_event_broadcasting() {
    let mut server = EditorServer::new().await;
    let client1_id = server.connect_client().await.unwrap();
    let client2_id = server.connect_client().await.unwrap();

    let buffer_id = server.create_buffer(client1_id, None).await.unwrap();

    // Subscribe both clients to buffer events
    server.subscribe_to_buffer(client1_id, buffer_id).await.unwrap();
    server.subscribe_to_buffer(client2_id, buffer_id).await.unwrap();

    // Make a change that should trigger events
    server.insert_char(buffer_id, 0, 'A').await.unwrap();

    // Both clients should receive the event
    let client1_events = server.get_client_events(client1_id).await.unwrap();
    let client2_events = server.get_client_events(client2_id).await.unwrap();

    assert!(!client1_events.is_empty());
    assert!(!client2_events.is_empty());

    // Check event content
    if let EditorEvent::BufferChanged { buffer_id: event_buffer_id, .. } = &client1_events[0] {
        assert_eq!(*event_buffer_id, buffer_id);
    } else {
        panic!("Expected BufferChanged event");
    }
}

#[tokio::test]
async fn test_modal_editing_states() {
    let mut server = EditorServer::new().await;
    let client_id = server.connect_client().await.unwrap();
    let buffer_id = server.create_buffer(client_id, None).await.unwrap();

    // Default mode should be Normal
    let mode = server.get_edit_mode(buffer_id).await.unwrap();
    assert_eq!(mode, EditMode::Normal);

    // Switch to Insert mode
    server.set_edit_mode(buffer_id, EditMode::Insert).await.unwrap();
    let mode = server.get_edit_mode(buffer_id).await.unwrap();
    assert_eq!(mode, EditMode::Insert);

    // Switch to Visual mode
    server.set_edit_mode(buffer_id, EditMode::Visual).await.unwrap();
    let mode = server.get_edit_mode(buffer_id).await.unwrap();
    assert_eq!(mode, EditMode::Visual);
}

#[tokio::test]
async fn test_mode_change_events() {
    let mut server = EditorServer::new().await;
    let client_id = server.connect_client().await.unwrap();
    let buffer_id = server.create_buffer(client_id, None).await.unwrap();

    server.subscribe_to_buffer(client_id, buffer_id).await.unwrap();

    server.set_edit_mode(buffer_id, EditMode::Insert).await.unwrap();

    let events = server.get_client_events(client_id).await.unwrap();
    let mode_events: Vec<_> = events.iter()
        .filter_map(|e| match e {
            EditorEvent::ModeChanged { mode, .. } => Some(mode),
            _ => None
        })
        .collect();

    assert!(!mode_events.is_empty());
    assert_eq!(*mode_events[0], EditMode::Insert);
}

#[tokio::test]
async fn test_concurrent_client_operations() {
    let server = std::sync::Arc::new(tokio::sync::Mutex::new(EditorServer::new().await));

    let handles: Vec<_> = (0..5).map(|i| {
        let server_clone = server.clone();
        tokio::spawn(async move {
            let mut server_guard = server_clone.lock().await;
            let client_id = server_guard.connect_client().await.unwrap();
            let buffer_id = server_guard.create_buffer(client_id, Some(format!("Buffer {}", i))).await.unwrap();
            (client_id, buffer_id)
        })
    }).collect();

    let results: Vec<_> = futures::future::join_all(handles).await;

    for result in results {
        let (client_id, buffer_id) = result.unwrap();
        let server_guard = server.lock().await;
        assert!(server_guard.is_client_connected(client_id));
        assert!(server_guard.get_buffer_content(buffer_id).await.is_ok());
    }

    let server_guard = server.lock().await;
    assert_eq!(server_guard.client_count(), 5);
    assert_eq!(server_guard.buffer_count(), 5);
}

#[tokio::test]
async fn test_error_handling() {
    let mut server = EditorServer::new().await;
    let client_id = server.connect_client().await.unwrap();

    // Try to operate on non-existent buffer
    let fake_buffer_id = BufferId::new();
    assert!(server.get_buffer_content(fake_buffer_id).await.is_err());
    assert!(server.insert_char(fake_buffer_id, 0, 'A').await.is_err());

    // Try to operate with non-existent client
    let fake_client_id = ClientId::new();
    assert!(server.create_buffer(fake_client_id, None).await.is_err());
}

#[tokio::test]
async fn test_buffer_cleanup_on_client_disconnect() {
    let mut server = EditorServer::new().await;
    let client_id = server.connect_client().await.unwrap();
    let buffer_id = server.create_buffer(client_id, None).await.unwrap();

    assert_eq!(server.buffer_count(), 1);

    server.disconnect_client(client_id).await.unwrap();

    // Buffer should be cleaned up when client disconnects
    assert_eq!(server.buffer_count(), 0);
    assert!(server.get_buffer_content(buffer_id).await.is_err());
}