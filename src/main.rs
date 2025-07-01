use std::io;
use crate::editor::Editor;

mod text_buffer;
mod editor;


fn main() -> io::Result<()> {
    let mut editor = Editor::new();

    match editor.run() {
        Ok(()) => {
            println!("editor exited successfully");
        }
        Err(e) => {
            println!("editor exited with error: {}", e);
        },
    }

    Ok(())
}
