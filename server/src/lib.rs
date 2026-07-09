use std::collections::HashMap;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::{error, info};

pub mod network_handler;

pub type IndexMap = HashMap<String, HashMap<PathBuf, Vec<usize>>>;

pub fn run_server(map: Arc<IndexMap>, case_sensitive: bool, addr: &str) -> std::io::Result<()> {
    let client_counter = Arc::new(AtomicUsize::new(1));
    let listener = TcpListener::bind(addr)?;
    info!("Server is active on {}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let map_clone = Arc::clone(&map);
                let counter_clone = Arc::clone(&client_counter);
                let client_id = counter_clone.fetch_add(1, Ordering::SeqCst);

                std::thread::spawn(move || {
                    if let Err(e) = network_handler::handle_network_client(
                        stream,
                        map_clone,
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
