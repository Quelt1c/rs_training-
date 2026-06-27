use std::collections::HashMap;
use std::fs;
use std::io;

fn print_char(map: &HashMap<char, Vec<usize>>)
{
    for(letter, list) in map
    {
        println!("{}: {:?}", letter, list);
    }
}

fn print_string(map: &HashMap<String, Vec<usize>>)
{
    for(word, list) in map
    {
        println!("{}: {:?}", word, list);
    }
}

fn position_indices(map: &mut HashMap<char, Vec<usize>>, text: &str)
{
    for(index, letter) in text.char_indices(){
        map.entry(letter).or_insert(Vec::new()).push(index)
    }
}

fn split_by_word(map: &mut HashMap<String, Vec<usize>>, text: &str)
{
    let mut current_index = 0;
    for word in text.split_whitespace()
    {
        map.entry(word.to_string()).or_insert(Vec::new()).push(current_index);
        current_index = current_index + word.len() + 1;
    }
}

fn split_by_word_own(map: &mut HashMap<String, Vec<usize>>, text: &str)
{
    let mut current_word = String::new();
    let mut start_index: Option<usize> = None;

    for(i, c) in text.char_indices()
    {
        if c.is_alphanumeric()
        {   
            if start_index.is_none()
            {
                start_index = Some(i);
            }
            current_word.push(c);
        }
        else
        {
            if let Some(ind) = start_index 
            {
                map.entry(current_word.clone()).or_insert(Vec::new()).push(ind);
                current_word.clear();
                start_index = None;
            }
        }
    }
    if let Some(ind) = start_index
    {
        map.entry(current_word).or_insert(Vec::new()).push(ind);
    }
}

fn write_in_variable() -> io::Result<String>
{
    let text_from_file = fs::read_to_string("file.txt")?;
    Ok(text_from_file)
}


fn main() -> io::Result<()> {

    
    let text = write_in_variable()?;
    for line in text.lines()
    {
        println!("{}", line);
    }
    
    /*
    let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
    let text_ukr = "Україна... В одному вже тільки цьому слові і для нашого вуха, і для вуха чужинців бринить ціла музика смутку і жалю... Україна — країна смутку і краси, країна, де найбільше люблять волю... Україна — це тихі води і ясні зорі, зелені сади, білі хати, лани золотої пшениці, медові та молочні ріки... Україна — розкішний вінок із рути і барвінку, що над ним світять заплакані золоті зорі... Поема жалю і смутку, краси і недолі...";
    */
    let mut count = HashMap::new();
    split_by_word_own(&mut count, &text);
    print_string(&count);
    Ok(())
}

    

