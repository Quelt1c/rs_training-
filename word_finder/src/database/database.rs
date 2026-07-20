use super::messages::DatabaseMessage;
use crate::channel;
use crate::io_utils::FileReport;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Database {
    sender: channel::Sender<DatabaseMessage>,
}

impl Database {
    pub fn new(output_rx: channel::Receiver<FileReport>) -> Self {
        let (tx, rx) = channel::unbounded();

        std::thread::spawn(move || {
            let mut storage: HashMap<String, HashMap<PathBuf, Vec<usize>>> = HashMap::new();

            while let Ok(report) = output_rx.recv() {
                for (word, indices) in report.words_map {
                    storage
                        .entry(word)
                        .or_default()
                        .insert(report.file_path.clone(), indices);
                }
            }

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
}
