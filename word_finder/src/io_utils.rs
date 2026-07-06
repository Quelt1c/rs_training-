use crate::text_tools::parser::split_by_word_own;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct FileReport {
    pub file_path: PathBuf,
    pub words_map: HashMap<String, Vec<usize>>,
}

pub fn walk_dir_single(
    path: &Path,
    map: &mut HashMap<String, HashMap<PathBuf, Vec<usize>>>,
    case_sensitive: bool,
) -> std::io::Result<()> {
    if path.is_file() {
        process_file_single(path, map, case_sensitive);
        return Ok(());
    }

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let current = entry.path();

        if current.is_dir() {
            walk_dir_single(&current, map, case_sensitive)?;
        } else {
            process_file_single(&current, map, case_sensitive);
        }
    }
    Ok(())
}

fn process_file_single(
    current: &Path,
    map: &mut HashMap<String, HashMap<PathBuf, Vec<usize>>>,
    case_sensitive: bool,
) {
    if current.is_file()
        && let Some(extension) = current.extension()
        && extension == "txt"
        && let Ok(text) = std::fs::read_to_string(current)
    {
        let mut wordt_map = HashMap::new();
        split_by_word_own(&mut wordt_map, &text);

        for (word, indices) in wordt_map {
            let processed_word = if case_sensitive {
                word
            } else {
                word.to_lowercase()
            };
            let word_entry = map.entry(processed_word).or_default();
            let file_indices = word_entry.entry(current.to_path_buf()).or_default();
            file_indices.extend(indices);
            file_indices.sort();
        }
    }
}

pub fn produce_file_tasks(path: &Path, input_tx: &flume::Sender<PathBuf>) -> std::io::Result<()> {
    if path.is_file() {
        let _ = input_tx.send(path.to_path_buf());
        return Ok(());
    }

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let current = entry.path();

        if current.is_dir() {
            produce_file_tasks(&current, input_tx)?;
        } else {
            let _ = input_tx.send(current);
        }
    }
    Ok(())
}

pub fn file_worker_channels(
    input_rx: flume::Receiver<PathBuf>,
    output_tx: flume::Sender<FileReport>,
    case_sensitive: bool,
) {
    while let Ok(current) = input_rx.recv() {
        if current.is_file()
            && let Some(extension) = current.extension()
            && extension == "txt"
            && let Ok(text) = std::fs::read_to_string(&current)
        {
            let mut wordt_map = HashMap::new();
            split_by_word_own(&mut wordt_map, &text);

            let mut processed_map: HashMap<String, Vec<usize>> = HashMap::new();

            for (word, indices) in wordt_map {
                let processed_word = if case_sensitive {
                    word
                } else {
                    word.to_lowercase()
                };
                let file_indices = processed_map.entry(processed_word).or_default();
                file_indices.extend(indices);
                file_indices.sort();
            }

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
