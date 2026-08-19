mod case_checker;
mod channel;
mod database;
mod server;
mod text_tools;
use crate::case_checker::Checker;
use anyhow;
use clap::Parser;
use database::Database;
use server::auth::generate_dev_password;
use tracing::{Level, info};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    let args: Checker = Checker::parse();

    info!("Working {} threads", args.threads);

    let threads = args.threads.get();
    let db = Database::new(args.file_path, threads, args.case_sensitive);

    let password = args.password.unwrap_or_else(generate_dev_password);
    info!(
        "Login for protected endpoints: username='{}' password='{}'",
        args.username, password
    );

    let credentials = server::auth::Credentials {
        username: args.username,
        password,
    };

    server::run_server("127.0.0.1:27015", db, credentials).await?;

    Ok(())
}
