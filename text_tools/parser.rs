use std::collections::HashMap;

pub fn position_indices(map: &mut HashMap<char, Vec<usize>>, text: &str) {
    for (index, letter) in text.char_indices() {
        map.entry(letter).or_insert_with(Vec::new).push(index);
    }
}

pub fn split_by_word_own(map: &mut HashMap<String, Vec<usize>>, text: &str) {
    let mut current_word = String::new();
    let mut start_index: Option<usize> = None;

    for (i, c) in text.char_indices() {
        if c.is_alphanumeric() {   
            if start_index.is_none() {
                start_index = Some(i);
            }
            current_word.push(c);
        } else if let Some(ind) = start_index {
            map.entry(current_word.clone()).or_insert_with(Vec::new).push(ind);
            current_word.clear();
            start_index = None;
        }
    }
    
    if let Some(ind) = start_index {
        map.entry(current_word).or_insert_with(Vec::new).push(ind);
    }
}

#[cfg(test)]
mod parser_tests;