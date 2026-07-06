use crate::text_tools::parser::split_by_word_own;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

type SharedMap = Arc<Mutex<HashMap<String, HashMap<PathBuf, Vec<usize>>>>>;

pub fn walk_dir(path: &Path, map: SharedMap, case_sensitive: bool) -> std::io::Result<()> {
    if path.is_file() {
        process_file(path, Arc::clone(&map), case_sensitive);
        return Ok(());
    }

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let current = entry.path();

        if current.is_dir() {
            walk_dir(&current, Arc::clone(&map), case_sensitive)?;
        } else {
            process_file(&current, Arc::clone(&map), case_sensitive);
        }
    }
    Ok(())
}

fn process_file(current: &Path, map: SharedMap, case_sensitive: bool) {
    if current.is_file()
        && let Some(extension) = current.extension()
        && extension == "txt"
        && let Ok(text) = std::fs::read_to_string(current)
    {
        let mut wordt_map = HashMap::new();
        split_by_word_own(&mut wordt_map, &text);

        let mut guard = map.lock().unwrap();

        for (word, indices) in wordt_map {
            let processed_word = if case_sensitive {
                word
            } else {
                word.to_lowercase()
            };
            let word_entry = guard.entry(processed_word).or_default();
            let file_indices = word_entry.entry(current.to_path_buf()).or_default();
            file_indices.extend(indices);
            file_indices.sort();
        }
    }
}
