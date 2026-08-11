use super::io_utils::{FileReport, file_worker_channels};
use crate::channel;
use std::path::PathBuf;
use tokio::task::JoinHandle;

pub fn spawn_worker_threads(
    input_rx: channel::Receiver<PathBuf>,
    output_tx: channel::Sender<FileReport>,
    case_sensitive: bool,
    threads: usize,
) -> Vec<JoinHandle<()>> {
    let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(threads);

    for _ in 0..threads {
        let input_rx_clone = input_rx.clone();
        let output_tx_clone = output_tx.clone();

        let handle = tokio::spawn(async move {
            file_worker_channels(input_rx_clone, output_tx_clone, case_sensitive).await;
        });

        handles.push(handle);
    }

    handles
}
