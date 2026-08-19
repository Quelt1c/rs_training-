// pub mod stream_server;

// use crate::database::Database;
// use anyhow::{Context, Result};
// use tokio::net::TcpListener;
// use tracing::{error, info};

// pub async fn run_server(addr: &str, db: Database) -> Result<()> {
//     let listener = TcpListener::bind(addr).await.with_context(|| {
//         format!("Failed to start the server. Check if port {addr} is already in use",)
//     })?;

//     info!("Server is running on http://{addr}");

//     let mut client_id = 0;

//     loop {
//         let (stream, _) = listener
//             .accept()
//             .await
//             .context("Critical error while attempting to accept a new network connection")?;

//         let db_clone = db.clone();
//         client_id += 1;

//         tokio::spawn(async move {
//             if let Err(e) = stream_server::handle_network_client(stream, db_clone, client_id).await
//             {
//                 error!("Client #{} error: {}", client_id, e);
//             }
//         });
//     }
// }

pub mod own_axum;
pub mod stream_server;

use crate::database::Database;
use anyhow::{Context, Result};
use own_axum::{Router, StatusCode, serve, with_query_handler};
use tokio::net::TcpListener;
use tracing::info;

pub async fn run_server(addr: &str, db: Database) -> Result<()> {
    let listener = TcpListener::bind(addr).await.with_context(|| {
        format!("Failed to start the server. Check if port {addr} is already in use")
    })?;

    info!("Server is running on http://{addr}");

    let router = Router::new()
        .route(
            "/search",
            with_query_handler(stream_server::search_handler, db.clone()),
        )
        .post(
            "/search",
            with_query_handler(stream_server::search_handler, db.clone()),
        )
        .route(
            "/download",
            with_query_handler(own_axum::download_handler, db.clone()),
        )
        .post(
            "/download",
            with_query_handler(own_axum::download_handler, db.clone()),
        )
        .route(
            "/info",
            with_query_handler(own_axum::info_handler, db.clone()),
        )
        .fallback(not_found);

    serve(listener, router).await
}

async fn not_found() -> (StatusCode, &'static str) {
    (
        StatusCode::NOT_FOUND,
        "404 Not Found. Please use format: /search?word=YOUR_WORD\n",
    )
}
