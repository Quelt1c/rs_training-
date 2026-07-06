use crate::text_tools::parser::split_by_word_own;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn walk_dir() {}

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
