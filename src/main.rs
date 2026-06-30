use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

mod io_utils;
mod text_tools;

fn main() -> std::io::Result<()> {
    let path = Path::new(".");

    let mut map = HashMap::<String, HashMap<PathBuf, Vec<usize>>>::new();

    io_utils::walk_dir(path, &mut map)?;

    loop {
        print!("Enter the word or '0' to exit: ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let word = input.trim();
        if word == "0" {
            println!("Exit");
            break;
        }
        if word.is_empty() {
            continue;
        }
        if let Some(files_with_word) = map.get(word) {
            println!("{word} found in files: ");
            for (file_path, indices) in files_with_word {
                println!(
                    "File: {:?} in the position: {:?}\n",
                    file_path.display(),
                    indices
                );
            }
        } else {
            println!("A word {word} not found.\n");
        }
    }

    Ok(())
}
