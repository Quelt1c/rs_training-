pub mod stream_server;

use crate::database::Database;
use anyhow::{Context, Result};
use tokio::net::TcpListener;
use tracing::{error, info};

pub async fn run_server(addr: &str, db: Database) -> Result<()> {
    let listener = TcpListener::bind(addr).await.with_context(|| {
        format!("Failed to start the server. Check if port {addr} is already in use",)
    })?;

    info!("Server is running on http://{addr}");

    let mut client_id = 0;

    loop {
        let (stream, _) = listener
            .accept()
            .await
            .context("Critical error while attempting to accept a new network connection")?;

        let db_clone = db.clone();
        client_id += 1;

        tokio::spawn(async move {
            if let Err(e) = stream_server::handle_network_client(stream, db_clone, client_id).await
            {
                error!("Client #{} error: {}", client_id, e);
            }
        });
    }
}
