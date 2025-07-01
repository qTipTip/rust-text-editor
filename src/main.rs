use crate::editor::Editor;
use std::io;

mod editor;
mod text_buffer;

fn main() -> io::Result<()> {
    let mut editor = Editor::new();
    match editor.run() {
        Ok(()) => {
            println!("editor exited successfully");
        }
        Err(e) => {
            println!("editor exited with error: {}", e);
        }
    }

    Ok(())
}
