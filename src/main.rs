use std::collections::HashMap;

mod io_utils;
mod text_tools;

use io_utils::read_file;
use text_tools::parser::split_by_word_own;
use text_tools::printer::print_string;

fn main() -> std::io::Result<()> {
    let text = read_file("file.txt")?;
    
    for line in text.lines() {
        println!("{}", line);
    }

    let mut count = HashMap::new();
    split_by_word_own(&mut count, &text);
    print_string(&count);
    
    Ok(())
}