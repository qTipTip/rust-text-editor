use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClientId(Uuid);

impl ClientId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BufferId(Uuid);

impl BufferId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EditMode {
    Normal,
    Insert,
    Visual,
    Command,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditorEvent {
    BufferChanged {
        buffer_id: BufferId,
        changes: Vec<TextChange>,
    },
    CursorMoved {
        buffer_id: BufferId,
        position: usize,
    },
    ModeChanged {
        buffer_id: BufferId,
        mode: EditMode,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextChange {
    pub position: usize,
    pub deleted_text: String,
    pub inserted_text: String,
}

#[derive(Debug)]
pub enum ServerError {
    ClientNotFound,
    BufferNotFound,
    InvalidOperation,
    InternalError(String),
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerError::ClientNotFound => write!(f, "Client not found"),
            ServerError::BufferNotFound => write!(f, "Buffer not found"),
            ServerError::InvalidOperation => write!(f, "Invalid operation"),
            ServerError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for ServerError {}

pub type ServerResult<T> = Result<T, ServerError>;
