mod case_checker;
mod channel;
mod database;
mod io_utils;
mod network_handler;
mod server;
mod spawn_workers;
mod text_tools;

use crate::case_checker::Checker;
use clap::Parser;
use database::Database;
use tracing::{Level, info};
#[cfg(test)]
mod channels_test;

fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    let args: Checker = Checker::parse();

    let db = Database::new();

    info!("Working {} threads", args.threads);

    let (input_tx, input_rx) = channel::unbounded();
    let (output_tx, output_rx) = channel::unbounded();

    spawn_workers::spawn_worker_threads(
        input_rx,
        output_tx,
        args.case_sensitive,
        args.threads.get(),
    );
    io_utils::produce_file_tasks(&args.file_path, input_tx)?;

    while let Ok(report) = output_rx.recv() {
        db.insert_report(report.file_path, report.words_map);
    }

    info!("Data indexing completed");

    server::run_server(db, args.case_sensitive, "127.0.0.1:27015")?;

    Ok(())
}
