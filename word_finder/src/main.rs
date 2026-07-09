use crate::case_checker::Checker;
use clap::Parser;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{Level, info};
mod case_checker;
mod io_utils;

mod spawn_workers;
mod text_tools;

fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let args: Checker = Checker::parse();
    let mut map: HashMap<String, HashMap<std::path::PathBuf, Vec<usize>>> = HashMap::new();

    if args.threads.get() == 1 {
        println!("working single thread");
        io_utils::walk_dir_single(&args.file_path, &mut map, args.case_sensitive)?;
    } else {
        println!("Working {} threads", args.threads);

        let (input_tx, input_rx) = flume::unbounded();
        let (output_tx, output_rx) = flume::unbounded();

        spawn_workers::spawn_worker_threads(
            input_rx,
            output_tx,
            args.case_sensitive,
            args.threads.get(),
        );

        io_utils::produce_file_tasks(&args.file_path, input_tx)?;

        while let Ok(report) = output_rx.recv() {
            for (word, indices) in report.words_map {
                map.entry(word)
                    .or_default()
                    .insert(report.file_path.clone(), indices);
            }
        }
    }

    info!("Data indexing complete. Starting network server...");

    let shared_map = Arc::new(map);
    server::run_server(shared_map, args.case_sensitive, "127.0.0.1:27015")?;

    Ok(())
}
