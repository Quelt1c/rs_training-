mod linkedlist;

use linkedlist::LinkedList;

fn main() {
    let mut list = LinkedList::new();
    list.push_front(1);
    list.push_front(2);
    list.push_front(3);

    for item in &list {
        print!("Elements: {}\n", item);
    }

    for item in &mut list {
        *item *= 2;
    }

    for item in &list {
        print!("After modification: {}\n", item);
    }
}
