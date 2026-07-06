use crate::case_checker::Checker;
use clap::Parser;
use std::collections::HashMap;
use std::io::{self, Write};

mod case_checker;
mod io_utils;
mod spawn_workers;
mod text_tools;

fn main() -> std::io::Result<()> {
    let args = Checker::parse();
    let mut map = HashMap::new();
    if args.threads == 1 {
        println!("working single thread");
        io_utils::walk_dir_single(&args.file_path, &mut map, args.case_sensitive)?;
    } else {
        println!("Working {} threads", args.threads);

        let (input_tx, input_rx) = flume::unbounded();
        let (output_tx, output_rx) = flume::unbounded();

        let handles = spawn_workers::spawn_worker_threads(
            input_rx,
            output_tx.clone(),
            args.case_sensitive,
            args.threads,
        );

        drop(output_tx);

        io_utils::produce_file_tasks(&args.file_path, &input_tx)?;

        drop(input_tx);

        while let Ok(report) = output_rx.recv() {
            for (word, indices) in report.words_map {
                let word_entry = map.entry(word).or_default();
                let file_indices = word_entry.entry(report.file_path.clone()).or_default();
                file_indices.extend(indices);
                file_indices.sort();
            }
        }

        for handle in handles {
            let _ = handle.join();
        }
    }
    loop {
        print!("Enter word or q to exit: ");
        io::stdout().flush()?;

        let mut input_word = String::new();
        io::stdin().read_line(&mut input_word)?;

        let trimmed_word = input_word.trim();

        if trimmed_word == "q" {
            println!("Exit");
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
            println!("{trimmed_word} found in files: ");
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
