use crate::channel;
use crate::io_utils::{FileReport, file_worker_channels};
use std::path::PathBuf;
use std::thread::{self, JoinHandle};

pub fn spawn_worker_threads(
    input_rx: channel::Receiver<PathBuf>,
    output_tx: channel::Sender<FileReport>,
    case_sensitive: bool,
    threads: usize,
) -> Vec<JoinHandle<()>> {
    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    for _ in 0..threads {
        let input_rx_clone: channel::Receiver<PathBuf> = input_rx.clone();
        let output_tx_clone: channel::Sender<FileReport> = output_tx.clone();

        let handle: JoinHandle<()> = thread::spawn(move || {
            file_worker_channels(input_rx_clone, output_tx_clone, case_sensitive);
        });

        handles.push(handle);
    }

    handles
}
