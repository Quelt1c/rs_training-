pub mod stream_server;
use crate::Database;
use std::net::TcpListener;
use tracing::{error, info};

pub fn run_server(db: Database, case_sensitive: bool, addr: &str) -> std::io::Result<()> {
    let mut client_counter: usize = 1;
    let listener = TcpListener::bind(addr)?;
    info!("Server is active on {}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let db_clone = db.clone();
                let client_id = client_counter;
                client_counter += 1;

                std::thread::spawn(move || {
                    if let Err(e) = stream_server::handle_network_client(
                        stream,
                        db_clone,
                        case_sensitive,
                        client_id,
                    ) {
                        error!(client_id = client_id, error = %e, "Error in client thread");
                    }
                });
            }
            Err(e) => error!(error = %e, "Incoming connection error"),
        }
    }
    Ok(())
}
