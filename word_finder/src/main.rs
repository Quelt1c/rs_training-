use crate::case_checker::Checker;
use clap::Parser;
use std::collections::HashMap;
use std::io::{self, Write};

mod case_checker;
mod io_utils;
mod spawn_threads;
mod text_tools;

fn main() -> std::io::Result<()> {
    let args = Checker::parse();
    let mut map = HashMap::new();

    io_utils::walk_dir_single(&args.file_path, &mut map, args.case_sensitive)?;

    loop {
        print!("Enter word to search (or type ':q' to exit): ");
        io::stdout().flush()?;

        let mut input_word = String::new();
        io::stdin().read_line(&mut input_word)?;

        let trimmed_word = input_word.trim();

        if trimmed_word == ":q" {
            println!("Exiting program. Bye!");
            break;
        }

        if trimmed_word.is_empty() {
            continue;
        }

        let search_word = if args.case_sensitive {
            trimmed_word.to_string()
        } else {
            trimmed_word.to_lowercase()
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
            println!("A word {trimmed_word} not found.\n");
        }
    }

    Ok(())
}
