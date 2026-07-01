use crate::text_tools::parser::split_by_word_own;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn walk_dir(
    path: &Path,
    map: &mut HashMap<String, HashMap<PathBuf, Vec<usize>>>,
) -> std::io::Result<()> {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let current = entry.path();

        if current.is_dir() {
            _ = walk_dir(&current, map)?;
        } else if current.is_file()
            && let Some(extension) = current.extension()
            && extension == "txt"
            && let Ok(text) = std::fs::read_to_string(&current)
        {
            let mut wordt_map = HashMap::new();
            split_by_word_own(&mut wordt_map, &text);

            for (word, indices) in wordt_map {
                map.entry(word)
                    .or_default()
                    .insert(current.clone(), indices);
            }
        }
    }
    Ok(())
}
