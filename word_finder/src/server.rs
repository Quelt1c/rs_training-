use crate::Database;
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::{error, info};

pub fn run_server(db: Database, case_sensitive: bool, addr: &str) -> std::io::Result<()> {
    let client_counter = Arc::new(AtomicUsize::new(1));
    let listener = TcpListener::bind(addr)?;
    info!("Server is active on {}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let db_clone = db.clone();
                let counter_clone = Arc::clone(&client_counter);
                let client_id = counter_clone.fetch_add(1, Ordering::SeqCst);

                std::thread::spawn(move || {
                    if let Err(e) = crate::network_handler::handle_network_client(
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
