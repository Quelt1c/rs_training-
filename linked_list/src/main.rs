mod linkedlist;

use linkedlist::LinkedList;

fn main() {
    let mut list = LinkedList::new();
    list.push_front(42);
    list.push_front(100);
    list.push_front(7);

    println!("Length of the list: {}", list.len());
    if list.is_empty() {
        println!("The list is empty.");
    } else {
        println!("The list is not empty.");
    }

    if let Some(&first) = list.peek_front() {
        println!("The first number without removing: {}", first);
    }

    for number in list {
        println!("Elements of the list: {}", number);
    }
}
