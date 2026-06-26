use std::collections::HashMap;

fn printHM(indices: &HashMap<char, Vec<usize>>)
{
    for (letter, list) in indices
    {
        println!("{}: {:?}", letter, list);
    }
}

fn main() {
    let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum vel nibh massa. Duis convallis elementum felis sit amet elementum. Donec vitae nibh sed lorem luctus consectetur. Fusce aliquam, eros sit amet elementum suscipit, lorem lorem ultricies nibh, sed malesuada augue orci nec dui. Phasellus gravida dolor in nunc pulvinar suscipit. Pellentesque nec molestie tellus. Pellentesque ut diam quis arcu sodales bibendum nec ut lorem. Suspendisse sagittis lacus metus, ut vestibulum quam lacinia a. Ut ac diam eu metus finibus pharetra eu quis erat. Phasellus cursus luctus tempus.";
    let mut count = HashMap::new();
    for (index, letter) in text.char_indices() {
        count.entry(letter).or_insert(Vec::new()).push(index);
    }
    printHM(&count);
}
