use crate::case_checker::Checker;
use clap::Parser;
use std::collections::HashMap;
use std::path::PathBuf;

mod case_checker;
mod io_utils;
mod text_tools;

fn main() -> std::io::Result<()> {
    let args = Checker::parse();

    let mut map = HashMap::<String, HashMap<PathBuf, Vec<usize>>>::new();

    io_utils::walk_dir(&args.file_path, &mut map, args.case_sensitive)?;

    let search_word = if args.case_sensitive {
        args.text
    } else {
        args.text.to_lowercase()
    };

    if let Some(files_with_word) = map.get(&search_word) {
        println!("{search_word} found in files: ");
        for (file_path, indices) in files_with_word {
            println!(
                "File: {:?} in the position: {:?}\n",
                file_path.display(),
                indices
            );
        }
    } else {
        println!("A word {search_word} not found.\n");
    }
    Ok(())
}
