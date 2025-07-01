use crate::editor::Editor;
use std::io;
use std::path::{Path, PathBuf};

mod editor;
mod text_buffer;
mod syntax;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let mut editor = if args.len() > 1 {
        let path = PathBuf::from(&args[1]);
        match Editor::open_file(path) {
            Ok(editor) => editor,
            Err(e) => {
                eprintln!("Error opening file: {} {}", &args[1], e);
                std::process::exit(1);
            }
        }
    } else {
        Editor::new()
    };

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
