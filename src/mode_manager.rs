use std::collections::HashMap;

pub enum Mode {
    Normal,
    Visual,
    Insert,
    Command
}
pub enum CommandAction {
    Save,
    SaveAs(String),
    Quit,
    ForceQuit,
    Open(String),
    GoToLine(usize),
    Search(String),
    Replace {from: String, to: String},
}

pub enum NormalModeAction {
    DeleteLine,
    YankLine,
    PutAfter,
    PutBefore,
    GoToTop,
    GoToBottom,
    DeleteChar,
    Undo,
    Redo,
}

pub struct ModeManager {
    current_mode: Mode,
    command_buffer: String,
    key_sequence: Vec<char>,
    commands: HashMap<String, CommandAction>
}