use crate::channel;
use crate::database::messages::DatabaseMessage;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
mod io_utils;
mod messages;
mod spawn_workers;

use io_utils::{FileReport, produce_file_tasks};
use spawn_workers::spawn_worker_threads;

#[derive(Debug, Clone, Serialize)]
pub struct IndexedFile {
    pub id: u64,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct WordMatch {
    pub file_id: u64,
    pub path: PathBuf,
    pub positions: Vec<usize>,
}
struct Storage {
    word_index: HashMap<String, HashMap<u64, Vec<usize>>>,
    file_index: HashMap<u64, HashMap<String, Vec<usize>>>,
    files: HashMap<u64, PathBuf>,
    path_to_id: HashMap<PathBuf, u64>,
    next_id: u64,
}

impl Storage {
    fn new() -> Self {
        Self {
            word_index: HashMap::new(),
            file_index: HashMap::new(),
            files: HashMap::new(),
            path_to_id: HashMap::new(),
            next_id: 1,
        }
    }

    fn merge_report(&mut self, report: FileReport) -> IndexedFile {
        let FileReport {
            file_path,
            words_map,
        } = report;

        let id = if let Some(&existing_id) = self.path_to_id.get(&file_path) {
            if let Some(old_words) = self.file_index.remove(&existing_id) {
                for word in old_words.keys() {
                    if let Some(file_map) = self.word_index.get_mut(word) {
                        file_map.remove(&existing_id);
                        if file_map.is_empty() {
                            self.word_index.remove(word);
                        }
                    }
                }
            }
            existing_id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            self.path_to_id.insert(file_path.clone(), id);
            self.files.insert(id, file_path.clone());
            id
        };

        for (word, positions) in &words_map {
            self.word_index
                .entry(word.clone())
                .or_default()
                .insert(id, positions.clone());
        }
        self.file_index.insert(id, words_map);

        IndexedFile {
            id,
            path: file_path,
        }
    }
}

async fn index_path(
    storage: &mut Storage,
    path: PathBuf,
    threads: usize,
    case_sensitive: bool,
) -> Result<Vec<IndexedFile>, String> {
    let (input_tx, input_rx) = channel::unbounded();
    let (output_tx, output_rx) = channel::unbounded();

    spawn_worker_threads(input_rx, output_tx, case_sensitive, threads);

    if let Err(e) = produce_file_tasks(path, input_tx).await {
        return Err(format!("Failed to scan path: {e}"));
    }

    let mut indexed = Vec::new();
    while let Ok(report) = output_rx.recv().await {
        indexed.push(storage.merge_report(report));
    }

    Ok(indexed)
}

#[derive(Clone)]
pub struct Database {
    sender: channel::Sender<DatabaseMessage>,
    case_sensitive: bool,
}

impl Database {
    pub fn new(file_path: PathBuf, threads: usize, case_sensitive: bool) -> Self {
        let (tx, rx) = channel::unbounded();

        tokio::spawn(async move {
            let mut storage = Storage::new();

            if let Err(e) = index_path(&mut storage, file_path, threads, case_sensitive).await {
                tracing::error!("File scanning error: {}", e);
            }

            while let Ok(msg) = rx.recv().await {
                match msg {
                    DatabaseMessage::SearchWord { word, respond_to } => {
                        let matches = storage
                            .word_index
                            .get(&word)
                            .map(|file_map| {
                                file_map
                                    .iter()
                                    .filter_map(|(id, positions)| {
                                        storage.files.get(id).map(|path| WordMatch {
                                            file_id: *id,
                                            path: path.clone(),
                                            positions: positions.clone(),
                                        })
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();
                        let _ = respond_to.send_async(matches).await;
                    }
                    DatabaseMessage::ListWords { respond_to } => {
                        let mut words: Vec<String> = storage.word_index.keys().cloned().collect();
                        words.sort();
                        let _ = respond_to.send_async(words).await;
                    }
                    DatabaseMessage::ListFiles { respond_to } => {
                        let mut files: Vec<IndexedFile> = storage
                            .files
                            .iter()
                            .map(|(id, path)| IndexedFile {
                                id: *id,
                                path: path.clone(),
                            })
                            .collect();
                        files.sort_by_key(|f| f.id);
                        let _ = respond_to.send_async(files).await;
                    }
                    DatabaseMessage::FileWords {
                        file_id,
                        respond_to,
                    } => {
                        let words = storage.file_index.get(&file_id).cloned();
                        let _ = respond_to.send_async(words).await;
                    }
                    DatabaseMessage::AddPath { path, respond_to } => {
                        let result = index_path(&mut storage, path, threads, case_sensitive).await;
                        let _ = respond_to.send_async(result).await;
                    }
                }
            }
        });

        Self {
            sender: tx,
            case_sensitive,
        }
    }

    fn normalize(&self, word: String) -> String {
        if self.case_sensitive {
            word
        } else {
            word.to_lowercase()
        }
    }

    pub async fn search_word(&self, word: String) -> Vec<WordMatch> {
        let (tx, rx) = channel::unbounded();

        let sent = self
            .sender
            .send_async(DatabaseMessage::SearchWord {
                word: self.normalize(word),
                respond_to: tx,
            })
            .await;

        if sent.is_err() {
            return Vec::new();
        }

        rx.recv().await.unwrap_or_default()
    }

    pub async fn list_words(&self) -> Vec<String> {
        let (tx, rx) = channel::unbounded();

        if self
            .sender
            .send_async(DatabaseMessage::ListWords { respond_to: tx })
            .await
            .is_err()
        {
            return Vec::new();
        }

        rx.recv().await.unwrap_or_default()
    }

    pub async fn list_files(&self) -> Vec<IndexedFile> {
        let (tx, rx) = channel::unbounded();

        if self
            .sender
            .send_async(DatabaseMessage::ListFiles { respond_to: tx })
            .await
            .is_err()
        {
            return Vec::new();
        }

        rx.recv().await.unwrap_or_default()
    }

    pub async fn file_words(&self, file_id: u64) -> Option<HashMap<String, Vec<usize>>> {
        let (tx, rx) = channel::unbounded();

        if self
            .sender
            .send_async(DatabaseMessage::FileWords {
                file_id,
                respond_to: tx,
            })
            .await
            .is_err()
        {
            return None;
        }

        rx.recv().await.ok().flatten()
    }

    pub async fn add_path(&self, path: PathBuf) -> Result<Vec<IndexedFile>, String> {
        let (tx, rx) = channel::unbounded();

        let sent = self
            .sender
            .send_async(DatabaseMessage::AddPath {
                path,
                respond_to: tx,
            })
            .await;

        if sent.is_err() {
            return Err("Database actor is not running".to_string());
        }

        match rx.recv().await {
            Ok(result) => result,
            Err(_) => Err("Database actor is not running".to_string()),
        }
    }
}
