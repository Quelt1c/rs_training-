use std::collections::HashMap;

pub fn split_by_word_own(map: &mut HashMap<String, Vec<usize>>, text: &str) {
    let mut current_word = String::new();
    let mut start_index: Option<usize> = None;

    for (i, c) in text.char_indices() {
        if c.is_alphanumeric() {
            if start_index.is_none() {
                start_index = Some(i);
            }
            current_word.push(c);
        } else if let Some(ind) = start_index.take() {
            let current_word = std::mem::take(&mut current_word);
            map.entry(current_word).or_insert_with(Vec::new).push(ind);
        }
    }

    if let Some(ind) = start_index {
        map.entry(current_word).or_insert_with(Vec::new).push(ind);
    }
}
