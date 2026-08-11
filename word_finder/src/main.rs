mod case_checker;
mod channel;
mod database;
mod server;
mod text_tools;
use crate::case_checker::Checker;
use anyhow;
use clap::Parser;
use database::Database;
use tracing::{Level, info};
#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    let args: Checker = Checker::parse();

    info!("Working {} threads", args.threads);

    let db = Database::new(args.file_path, args.threads.get(), args.case_sensitive);

    server::run_server("127.0.0.1:27015", db).await?;

    Ok(())
}
