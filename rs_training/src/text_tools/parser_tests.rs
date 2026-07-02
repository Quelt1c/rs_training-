use crate::text_tools::parser::split_by_word_own;
use std::collections::HashMap;

const LOREM_TEXT: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.";
const UKR_TEXT: &str = "Україна... В одному вже тільки цьому слові бринить музика смутку і жалю...";

#[test]
fn test_general_text_first_word() {
    let mut map = HashMap::new();
    split_by_word_own(&mut map, LOREM_TEXT);
    assert_eq!(map.get("Lorem"), Some(&vec![0]));
}

#[test]
fn test_general_text_punctuation_handling() {
    let mut map = HashMap::new();
    split_by_word_own(&mut map, LOREM_TEXT);
    assert_eq!(map.get("amet"), Some(&vec![22]));
}

#[test]
fn test_general_text_word_existence() {
    let mut map = HashMap::new();
    split_by_word_own(&mut map, LOREM_TEXT);
    assert!(map.get("consectetur").is_some());
    assert!(map.get("aliqua").is_some());
}

#[test]
fn test_general_text_missing_word() {
    let mut map = HashMap::new();
    split_by_word_own(&mut map, LOREM_TEXT);
    assert_eq!(map.get("Rust"), None);
}

#[test]
fn test_general_text_case_sensitivity() {
    let mut map = HashMap::new();
    split_by_word_own(&mut map, LOREM_TEXT);
    assert_eq!(map.get("lorem"), None);
}

#[test]
fn test_ukrainian_general_text() {
    let mut map = HashMap::new();
    split_by_word_own(&mut map, UKR_TEXT);
    assert_eq!(map.get("Україна"), Some(&vec![0]));
    assert!(map.get("музика").is_some());
    assert!(map.get("жалю").is_some());
}

#[test]
fn test_multiple_occurrences() {
    let mut map = HashMap::new();
    let repeating_text = "word text word text word";
    split_by_word_own(&mut map, repeating_text);
    assert_eq!(map.get("word"), Some(&vec![0, 10, 20]));
}
