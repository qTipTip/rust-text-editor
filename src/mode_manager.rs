use ropey::str_utils::line_to_byte_idx;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    Visual,
    Insert,
    Command,
}

impl Mode {
    pub fn name(&self) -> &'static str {
        match self {
            Mode::Normal => "NORMAL",
            Mode::Visual => "VISUAL",
            Mode::Insert => "INSERT",
            Mode::Command => "COMMAND",
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum CommandAction {
    Save,
    SaveAs(String),
    Quit,
    ForceQuit,
    Open(String),
    GoToLine(usize),
    Search(String),
    Replace { from: String, to: String },
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct ModeManager {
    current_mode: Mode,
    command_buffer: String,
    key_sequence: Vec<char>,
    commands: HashMap<String, CommandAction>,
}

impl ModeManager {
    pub fn new() -> ModeManager {
        let mut commands = HashMap::new();

        commands.insert("w".to_string(), CommandAction::Save);
        commands.insert("q".to_string(), CommandAction::Quit);
        commands.insert("q!".to_string(), CommandAction::ForceQuit);
        commands.insert("wq".to_string(), CommandAction::Save);

        Self {
            current_mode: Mode::Normal,
            command_buffer: String::new(),
            key_sequence: Vec::new(),
            commands,
        }
    }
    pub fn current_mode(&self) -> &Mode {
        &self.current_mode
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.current_mode = mode;
        self.clear_command_buffer();
        self.clear_key_sequence();
    }

    pub fn add_to_command_buffer(&mut self, ch: char) {
        self.command_buffer.push(ch)
    }

    pub fn clear_command_buffer(&mut self) {
        self.command_buffer.clear();
    }

    pub fn backspace_command_buffer(&mut self) {
        self.command_buffer.pop();
    }

    pub fn command_buffer(&self) -> &str {
        &self.command_buffer
    }

    pub fn execute_command(&mut self) -> Option<CommandAction> {
        let cmd = self.command_buffer.trim();

        if cmd.starts_with("e ") {
            let filename = cmd[2..].trim().to_string();
            self.clear_command_buffer();
            return Some(CommandAction::Open(filename));
        }

        if cmd.starts_with("w ") {
            let filename = cmd[2..].trim().to_string();
            self.clear_command_buffer();
            return Some(CommandAction::SaveAs(filename));
        }

        if let Ok(line_num) = cmd.parse::<usize>() {
            self.clear_command_buffer();
            return Some(CommandAction::GoToLine(line_num));
        }

        if cmd.starts_with("/") {
            let search_term = cmd[1..].to_string();
            self.clear_command_buffer();
            return Some(CommandAction::Search(search_term));
        }

        if cmd.starts_with("%s/") {
            let parts: Vec<&str> = cmd[3..].split('/').collect();
            if parts.len() >= 2 {
                let from = parts[0].to_string();
                let to = parts[1].to_string();

                self.clear_command_buffer();
                return Some(CommandAction::Replace{from, to})
            }
        }

        if let Some(command) = self.commands.get(cmd) {
            let action = command.clone();
            self.clear_command_buffer();
            return Some(action)
        }
        self.clear_command_buffer();
        None
    }

    pub fn add_key_to_sequence(&mut self, key: char) {
        self.key_sequence.push(key);
    }

    pub fn clear_key_sequence(&mut self) {
        self.key_sequence.clear();
    }

    pub fn get_key_sequence(&self) -> &[char] {
        &self.key_sequence
    }

    pub fn process_normal_mode_keys(&mut self) -> Option<NormalModeAction> {
        let sequence: String = self.key_sequence.iter().collect();

        match sequence.as_str() {
            "dd" => {
                self.clear_key_sequence();
                Some(NormalModeAction::DeleteLine)
            },
            "yy" => {
                self.clear_key_sequence();
                Some(NormalModeAction::YankLine)
            },
            "p" => {
                self.clear_key_sequence();
                Some(NormalModeAction::PutAfter)
            },
            "P" => {
                self.clear_key_sequence();
                Some(NormalModeAction::PutBefore)
            },
            "gg" => {
                self.clear_key_sequence();
                Some(NormalModeAction::GoToTop)
            },
            "G" => {
                self.clear_key_sequence();
                Some(NormalModeAction::GoToBottom)
            },
            "x" => {
                self.clear_key_sequence();
                Some(NormalModeAction::DeleteChar)
            },
            "u" => {
                self.clear_key_sequence();
                Some(NormalModeAction::Undo)
            },
            "r" if self.key_sequence.len() == 1 => {
                None
            },
            sequence if sequence.starts_with("r") && sequence.len() == 2 => {
                self.clear_key_sequence();
                Some(NormalModeAction::Redo) // TODO: Make this a replace-action instead
            },
            _ => {
                if self.key_sequence.len() > 3 {
                    self.clear_key_sequence();
                }
                None
            }
        }
    }

    pub fn is_sequence_pending(&self) -> bool {
        if self.key_sequence.is_empty() {
            return false;
        }

        let sequence: String = self.key_sequence.iter().collect();

        match sequence.as_str() {
            "d" | "g" | "r" => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_creation() {
        let mode_manager = ModeManager::new();
        assert_eq!(mode_manager.current_mode(), &Mode::Normal);
        assert_eq!(mode_manager.command_buffer(), "");
        assert!(mode_manager.get_key_sequence().is_empty());
    }

    #[test]
    fn test_mode_switching() {
        let mut mode_manager = ModeManager::new();

        // Normal -> Insert
        mode_manager.set_mode(Mode::Insert);
        assert_eq!(mode_manager.current_mode(), &Mode::Insert);

        // Insert -> Normal
        mode_manager.set_mode(Mode::Normal);
        assert_eq!(mode_manager.current_mode(), &Mode::Normal);

        // Normal -> Visual
        mode_manager.set_mode(Mode::Visual);
        assert_eq!(mode_manager.current_mode(), &Mode::Visual);

        // Visual -> Command
        mode_manager.set_mode(Mode::Command);
        assert_eq!(mode_manager.current_mode(), &Mode::Command);
    }

    #[test]
    fn test_mode_names() {
        assert_eq!(Mode::Normal.name(), "NORMAL");
        assert_eq!(Mode::Insert.name(), "INSERT");
        assert_eq!(Mode::Visual.name(), "VISUAL");
        assert_eq!(Mode::Command.name(), "COMMAND");
    }

    #[test]
    fn test_command_buffer_operations() {
        let mut mode_manager = ModeManager::new();

        // Add characters
        mode_manager.add_to_command_buffer('w');
        assert_eq!(mode_manager.command_buffer(), "w");

        mode_manager.add_to_command_buffer('q');
        assert_eq!(mode_manager.command_buffer(), "wq");

        // Backspace
        mode_manager.backspace_command_buffer();
        assert_eq!(mode_manager.command_buffer(), "w");

        // Clear
        mode_manager.clear_command_buffer();
        assert_eq!(mode_manager.command_buffer(), "");
    }

    #[test]
    fn test_simple_command_execution() {
        let mut mode_manager = ModeManager::new();

        // Test save command
        mode_manager.add_to_command_buffer('w');
        let action = mode_manager.execute_command();
        assert_eq!(action, Some(CommandAction::Save));
        assert_eq!(mode_manager.command_buffer(), ""); // Should be cleared

        // Test quit command
        mode_manager.add_to_command_buffer('q');
        let action = mode_manager.execute_command();
        assert_eq!(action, Some(CommandAction::Quit));

        // Test force quit
        for ch in "q!".chars() {
            mode_manager.add_to_command_buffer(ch);
        }
        let action = mode_manager.execute_command();
        assert_eq!(action, Some(CommandAction::ForceQuit));
    }

    #[test]
    fn test_goto_line_command() {
        let mut mode_manager = ModeManager::new();

        // Test goto line 42
        for ch in "42".chars() {
            mode_manager.add_to_command_buffer(ch);
        }
        let action = mode_manager.execute_command();
        assert_eq!(action, Some(CommandAction::GoToLine(42)));

        // Test goto line 1
        mode_manager.add_to_command_buffer('1');
        let action = mode_manager.execute_command();
        assert_eq!(action, Some(CommandAction::GoToLine(1)));
    }

    #[test]
    fn test_search_command() {
        let mut mode_manager = ModeManager::new();

        // Test search
        for ch in "/hello".chars() {
            mode_manager.add_to_command_buffer(ch);
        }
        let action = mode_manager.execute_command();
        assert_eq!(action, Some(CommandAction::Search("hello".to_string())));

        // Test empty search
        mode_manager.add_to_command_buffer('/');
        let action = mode_manager.execute_command();
        assert_eq!(action, Some(CommandAction::Search("".to_string())));
    }

    #[test]
    fn test_file_commands() {
        let mut mode_manager = ModeManager::new();

        // Test open file
        for ch in "e test.txt".chars() {
            mode_manager.add_to_command_buffer(ch);
        }
        let action = mode_manager.execute_command();
        assert_eq!(action, Some(CommandAction::Open("test.txt".to_string())));

        // Test save as
        for ch in "w newfile.rs".chars() {
            mode_manager.add_to_command_buffer(ch);
        }
        let action = mode_manager.execute_command();
        assert_eq!(
            action,
            Some(CommandAction::SaveAs("newfile.rs".to_string()))
        );
    }

    #[test]
    fn test_replace_command() {
        let mut mode_manager = ModeManager::new();

        // Test replace
        for ch in "%s/old/new/".chars() {
            mode_manager.add_to_command_buffer(ch);
        }
        let action = mode_manager.execute_command();
        assert_eq!(
            action,
            Some(CommandAction::Replace {
                from: "old".to_string(),
                to: "new".to_string()
            })
        );

        // Test replace with empty strings
        for ch in "%s//replacement/".chars() {
            mode_manager.add_to_command_buffer(ch);
        }
        let action = mode_manager.execute_command();
        assert_eq!(
            action,
            Some(CommandAction::Replace {
                from: "".to_string(),
                to: "replacement".to_string()
            })
        );
    }

    #[test]
    fn test_invalid_commands() {
        let mut mode_manager = ModeManager::new();

        // Test unknown command
        for ch in "unknown".chars() {
            mode_manager.add_to_command_buffer(ch);
        }
        let action = mode_manager.execute_command();
        assert_eq!(action, None);
        assert_eq!(mode_manager.command_buffer(), ""); // Should still be cleared

        // Test invalid number
        for ch in "abc".chars() {
            mode_manager.add_to_command_buffer(ch);
        }
        let action = mode_manager.execute_command();
        assert_eq!(action, None);
    }

    #[test]
    fn test_normal_mode_key_sequences() {
        let mut mode_manager = ModeManager::new();

        // Test single character commands
        mode_manager.add_key_to_sequence('x');
        let action = mode_manager.process_normal_mode_keys();
        assert_eq!(action, Some(NormalModeAction::DeleteChar));
        assert!(mode_manager.get_key_sequence().is_empty()); // Should be cleared

        mode_manager.add_key_to_sequence('u');
        let action = mode_manager.process_normal_mode_keys();
        assert_eq!(action, Some(NormalModeAction::Undo));

        mode_manager.add_key_to_sequence('p');
        let action = mode_manager.process_normal_mode_keys();
        assert_eq!(action, Some(NormalModeAction::PutAfter));

        mode_manager.add_key_to_sequence('P');
        let action = mode_manager.process_normal_mode_keys();
        assert_eq!(action, Some(NormalModeAction::PutBefore));
    }

    #[test]
    fn test_two_character_sequences() {
        let mut mode_manager = ModeManager::new();

        // Test 'dd' (delete line)
        mode_manager.add_key_to_sequence('d');
        let action = mode_manager.process_normal_mode_keys();
        assert_eq!(action, None); // First 'd' doesn't complete action
        assert!(!mode_manager.get_key_sequence().is_empty()); // Sequence should remain

        mode_manager.add_key_to_sequence('d');
        let action = mode_manager.process_normal_mode_keys();
        assert_eq!(action, Some(NormalModeAction::DeleteLine));
        assert!(mode_manager.get_key_sequence().is_empty()); // Should be cleared

        // Test 'yy' (yank line)
        mode_manager.add_key_to_sequence('y');
        assert_eq!(mode_manager.process_normal_mode_keys(), None);

        mode_manager.add_key_to_sequence('y');
        let action = mode_manager.process_normal_mode_keys();
        assert_eq!(action, Some(NormalModeAction::YankLine));

        // Test 'gg' (go to top)
        mode_manager.add_key_to_sequence('g');
        assert_eq!(mode_manager.process_normal_mode_keys(), None);

        mode_manager.add_key_to_sequence('g');
        let action = mode_manager.process_normal_mode_keys();
        assert_eq!(action, Some(NormalModeAction::GoToTop));
    }

    #[test]
    fn test_sequence_pending() {
        let mut mode_manager = ModeManager::new();

        // No sequence started
        assert!(!mode_manager.is_sequence_pending());

        // Start a sequence
        mode_manager.add_key_to_sequence('d');
        assert!(mode_manager.is_sequence_pending());

        // Complete the sequence
        mode_manager.add_key_to_sequence('d');
        mode_manager.process_normal_mode_keys();
        assert!(!mode_manager.is_sequence_pending());

        // Invalid sequence
        mode_manager.add_key_to_sequence('z');
        assert!(!mode_manager.is_sequence_pending());
    }

    #[test]
    fn test_long_invalid_sequence_clearing() {
        let mut mode_manager = ModeManager::new();

        // Add too many characters
        mode_manager.add_key_to_sequence('a');
        mode_manager.add_key_to_sequence('b');
        mode_manager.add_key_to_sequence('c');
        mode_manager.add_key_to_sequence('d');

        // Should clear the sequence
        let action = mode_manager.process_normal_mode_keys();
        assert_eq!(action, None);
        assert!(mode_manager.get_key_sequence().is_empty());
    }

    #[test]
    fn test_mode_switching_clears_state() {
        let mut mode_manager = ModeManager::new();

        // Set up some state
        mode_manager.add_to_command_buffer('w');
        mode_manager.add_key_to_sequence('d');

        // Switch mode
        mode_manager.set_mode(Mode::Insert);

        // State should be cleared
        assert_eq!(mode_manager.command_buffer(), "");
        assert!(mode_manager.get_key_sequence().is_empty());
    }

    #[test]
    fn test_redo_sequence() {
        let mut mode_manager = ModeManager::new();

        // Test 'r' followed by another character (redo)
        mode_manager.add_key_to_sequence('r');
        assert_eq!(mode_manager.process_normal_mode_keys(), None);
        assert!(mode_manager.is_sequence_pending());

        mode_manager.add_key_to_sequence('x'); // Any character after 'r'
        let action = mode_manager.process_normal_mode_keys();
        assert_eq!(action, Some(NormalModeAction::Redo));
        assert!(mode_manager.get_key_sequence().is_empty());
    }

    #[test]
    fn test_capital_g_goto_bottom() {
        let mut mode_manager = ModeManager::new();

        mode_manager.add_key_to_sequence('G');
        let action = mode_manager.process_normal_mode_keys();
        assert_eq!(action, Some(NormalModeAction::GoToBottom));
        assert!(mode_manager.get_key_sequence().is_empty());
    }
}
