use crate::case_checker::Checker;
use clap::Parser;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod case_checker;
mod io_utils;
mod text_tools;
mod threads_create;

fn main() -> std::io::Result<()> {
    let args = Checker::parse();

    let shared_map = Arc::new(Mutex::new(HashMap::new()));

    let handles = threads_create::spawn_search_threads(
        &args.file_path,
        Arc::clone(&shared_map),
        args.case_sensitive,
    )?;

    println!("Threads are launched: {}.", handles.len());

    for handle in handles {
        handle
            .join()
            .expect("One of the threads finished with an error");
    }

    let map = Arc::try_unwrap(shared_map)
        .expect("Error: Arc is still being held by some thread")
        .into_inner()
        .expect("Error: Mutex is in a poisoned state");

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
