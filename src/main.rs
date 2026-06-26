use std::collections::HashMap;

fn print_char(indices: &HashMap<char, Vec<usize>>)
{
    for(letter, list) in indices
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

fn main() {
    let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum vel nibh massa. Duis convallis elementum felis sit amet elementum. Donec vitae nibh sed lorem luctus consectetur. Fusce aliquam, eros sit amet elementum suscipit, lorem lorem ultricies nibh, sed malesuada augue orci nec dui. Phasellus gravida dolor in nunc pulvinar suscipit. Pellentesque nec molestie tellus. Pellentesque ut diam quis arcu sodales bibendum nec ut lorem. Suspendisse sagittis lacus metus, ut vestibulum quam lacinia a. Ut ac diam eu metus finibus pharetra eu quis erat. Phasellus cursus luctus tempus.";
    //let count = HashMap::new();
    //position_indices(&count);
    let mut count_word = HashMap::new();
    split_by_word(&mut count_word, text);
    print_string(&count_word);

}
