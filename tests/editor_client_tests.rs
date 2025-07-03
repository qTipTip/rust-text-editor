use rust_text_editor::client::editor_client::EditorClient;
use rust_text_editor::server::events::{BufferId, EditMode};

#[tokio::test]
async fn test_client_creation() {
    let client = EditorClient::new().await.unwrap();
    assert_eq!(client.buffer_count(), 0);
}

#[tokio::test]
async fn test_create_buffer() {
    let mut client = EditorClient::new().await.unwrap();

    let buffer_id = client.create_buffer(None).await.unwrap();
    assert_eq!(client.buffer_count(), 1);

    let content = client.get_content(buffer_id).await.unwrap();
    assert_eq!(content, "");
}

#[tokio::test]
async fn test_create_buffer_with_content() {
    let mut client = EditorClient::new().await.unwrap();

    let buffer_id = client
        .create_buffer(Some("hello world".to_string()))
        .await
        .unwrap();
    let content = client.get_content(buffer_id).await.unwrap();
    assert_eq!(content, "hello world");
}

#[tokio::test]
async fn test_basic_editing() {
    let mut client = EditorClient::new().await.unwrap();
    let buffer_id = client.create_buffer(Some("abc".to_string())).await.unwrap();

    // Insert character at position
    client.insert_char(buffer_id, 1, 'X').await.unwrap();
    let content = client.get_content(buffer_id).await.unwrap();
    assert_eq!(content, "aXbc");

    // Delete character
    client.delete_char(buffer_id, 1).await.unwrap();
    let content = client.get_content(buffer_id).await.unwrap();
    assert_eq!(content, "abc");
}

#[tokio::test]
async fn test_cursor_operations() {
    let mut client = EditorClient::new().await.unwrap();
    let buffer_id = client
        .create_buffer(Some("hello".to_string()))
        .await
        .unwrap();

    // Test cursor position
    client.set_cursor_position(buffer_id, 2).await.unwrap();
    let pos = client.get_cursor_position(buffer_id).await.unwrap();
    assert_eq!(pos, 2);

    // Test cursor movement
    client.move_cursor_right(buffer_id).await.unwrap();
    let pos = client.get_cursor_position(buffer_id).await.unwrap();
    assert_eq!(pos, 3);
}

#[tokio::test]
async fn test_mode_operations() {
    let mut client = EditorClient::new().await.unwrap();
    let buffer_id = client.create_buffer(None).await.unwrap();

    // Default mode should be Normal
    let mode = client.get_mode(buffer_id).await.unwrap();
    assert_eq!(mode, EditMode::Normal);

    // Switch to Insert mode
    client.set_mode(buffer_id, EditMode::Insert).await.unwrap();
    let mode = client.get_mode(buffer_id).await.unwrap();
    assert_eq!(mode, EditMode::Insert);
}

#[tokio::test]
async fn test_multiple_buffers() {
    let mut client = EditorClient::new().await.unwrap();

    let buffer1 = client
        .create_buffer(Some("buffer1".to_string()))
        .await
        .unwrap();
    let buffer2 = client
        .create_buffer(Some("buffer2".to_string()))
        .await
        .unwrap();

    assert_eq!(client.buffer_count(), 2);

    let content1 = client.get_content(buffer1).await.unwrap();
    let content2 = client.get_content(buffer2).await.unwrap();

    assert_eq!(content1, "buffer1");
    assert_eq!(content2, "buffer2");
}

#[tokio::test]
async fn test_error_handling() {
    let mut client = EditorClient::new().await.unwrap();

    // Try to operate on non-existent buffer
    let fake_buffer_id = BufferId::new();
    assert!(client.get_content(fake_buffer_id).await.is_err());
    assert!(client.insert_char(fake_buffer_id, 0, 'X').await.is_err());
}
