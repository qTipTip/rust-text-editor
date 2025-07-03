use crate::editor::Editor;
use std::io;
use std::path::{Path, PathBuf};

mod client;
mod editor;
mod server;
mod syntax;
mod text_buffer;
#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let mut editor = if args.len() > 1 {
        let path = PathBuf::from(&args[1]);
        match Editor::open_file(path).await {
            Ok(editor) => editor,
            Err(e) => {
                eprintln!("Error opening file: {} {:?}", &args[1], e);
                std::process::exit(1);
            }
        }
    } else {
        match Editor::new().await {
            Ok(editor) => editor,
            Err(e) => {
                eprintln!("Error creating editor file: {} {:?}", &args[1], e);
                std::process::exit(1);
            }
        }
    };

    // match editor.run().await {
    //     Ok(()) => {
    //         println!("editor exited successfully");
    //     }
    //     Err(e) => {
    //         println!("editor exited with error: {}", e);
    //     }
    // }

    Ok(())
}
