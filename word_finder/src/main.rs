mod case_checker;
mod channel;
mod database;
mod pipeline;
use crate::server::network_handler;
mod server;
mod text_tools;
use crate::case_checker::Checker;
use clap::Parser;
use database::Database;
use pipeline::io_utils;
use pipeline::spawn_workers;
use tracing::{Level, info};

fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    let args: Checker = Checker::parse();

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

    let db = Database::new(output_rx);

    info!("Database initialized, indexing in background...");

    server::run_server(db, args.case_sensitive, "127.0.0.1:27015")?;

    Ok(())
}
