use super::messages::DatabaseMessage;
use crate::channel;
use crate::text_tools::parser::split_by_word_own;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::thread::{self, JoinHandle};

struct FileReport {
    pub file_path: PathBuf,
    pub words_map: HashMap<String, Vec<usize>>,
}

#[derive(Clone)]
pub struct Database {
    sender: channel::Sender<DatabaseMessage>,
}

impl Database {
    pub fn new(file_path: PathBuf, threads: usize, case_sensitive: bool) -> Self {
        let (tx, rx) = channel::unbounded();

        std::thread::spawn(move || {
            let mut storage: HashMap<String, HashMap<PathBuf, Vec<usize>>> = HashMap::new();

            let (input_tx, input_rx) = channel::unbounded();
            let (output_tx, output_rx) = channel::unbounded();

            Self::spawn_worker_threads(input_rx, output_tx, case_sensitive, threads);

            if let Err(e) = Self::produce_file_tasks(&file_path, input_tx) {
                tracing::error!("File scanning error: {}", e);
            }

            while let Ok(report) = output_rx.recv() {
                let FileReport {
                    file_path,
                    words_map,
                } = report;

                for (word, indices) in words_map {
                    storage
                        .entry(word)
                        .or_default()
                        .insert(file_path.clone(), indices);
                }
            }

            tracing::info!("Background indexing of the database completed successfully!");

            while let Ok(msg) = rx.recv() {
                match msg {
                    DatabaseMessage::Search { word, respond_to } => {
                        let res = storage.get(&word).cloned();
                        let _ = respond_to.send((word, res));
                    }
                }
            }
        });

        Self { sender: tx }
    }

    pub fn get(&self, word: String) -> Option<HashMap<PathBuf, Vec<usize>>> {
        let (response_tx, response_rx) = channel::unbounded();

        let send_result = self.sender.send(DatabaseMessage::Search {
            word,
            respond_to: response_tx,
        });

        if send_result.is_err() {
            return None;
        }
        match response_rx.recv() {
            Ok((_returned_word, result)) => result,
            Err(_) => None,
        }
    }

    fn spawn_worker_threads(
        input_rx: channel::Receiver<PathBuf>,
        output_tx: channel::Sender<FileReport>,
        case_sensitive: bool,
        threads: usize,
    ) -> Vec<JoinHandle<()>> {
        let mut handles: Vec<JoinHandle<()>> = Vec::new();

        for _ in 0..threads {
            let input_rx_clone = input_rx.clone();
            let output_tx_clone = output_tx.clone();

            let handle = thread::spawn(move || {
                Self::file_worker_channels(input_rx_clone, output_tx_clone, case_sensitive);
            });

            handles.push(handle);
        }

        handles
    }

    fn produce_file_tasks(path: &Path, input_tx: channel::Sender<PathBuf>) -> std::io::Result<()> {
        if path.is_file() {
            let _ = input_tx.send(path.to_path_buf());
            return Ok(());
        }

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let current = entry.path();

            if current.is_dir() {
                Self::produce_file_tasks(&current, input_tx.clone())?;
            } else {
                let _ = input_tx.send(current);
            }
        }
        Ok(())
    }

    fn file_worker_channels(
        input_rx: channel::Receiver<PathBuf>,
        output_tx: channel::Sender<FileReport>,
        case_sensitive: bool,
    ) {
        while let Ok(current) = input_rx.recv() {
            if let Some(processed_map) = Self::parse_and_normalize_file(&current, case_sensitive) {
                let report = FileReport {
                    file_path: current,
                    words_map: processed_map,
                };

                if output_tx.send(report).is_err() {
                    break;
                }
            }
        }
    }

    fn parse_and_normalize_file(
        current: &Path,
        case_sensitive: bool,
    ) -> Option<HashMap<String, Vec<usize>>> {
        if current.is_file()
            && current.extension().map_or(false, |ext| ext == "txt")
            && let Ok(text) = std::fs::read_to_string(current)
        {
            let mut raw_map = HashMap::new();
            split_by_word_own(&mut raw_map, &text);

            let mut processed_map: HashMap<String, Vec<usize>> =
                HashMap::with_capacity(raw_map.len());

            for (word, indices) in raw_map {
                let processed_word = if case_sensitive {
                    word
                } else {
                    word.to_lowercase()
                };
                processed_map.insert(processed_word, indices);
            }
            return Some(processed_map);
        }
        None
    }
}
