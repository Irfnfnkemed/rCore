//#![no_std]

extern crate alloc;

use crate::linked_list::DoublyLinkedList;

mod block_cache;
mod block_dev;
mod linked_list;

struct A {
    x: usize,
}


fn main() {
    // let mut list = DoublyLinkedList::new();
    // println!("ddddd");
    // list.push_front(1, A { x: 11 });
    // list.push_front(2, A { x: 22 });
    // list.push_front(3, A { x: 33 });
    //
    // println!("push 1: {:?}", list.push_front(9, A { x: 99 }));
    // println!("Pop back: {:?}", list.pop_back().unwrap().1.x);
    // println!("push 1: {:?}", list.push_front(6, A { x: 66 }));
    // println!("push 1: {:?}", list.move_to_head(3));
    // let (k, v) = list.find(6).unwrap();
    // println!("find {:?},{:?}", k, v.x);
    // let (k, v) = list.find(6).unwrap();
    // println!("find {:?},{:?}", k, v.x);
    // println!("Pop back: {:?}", list.pop_back().unwrap().1.x);
    // println!("Pop back: {:?}", list.pop_back().unwrap().1.x);
    // println!("Pop back: {:?}", list.pop_back().unwrap().1.x);
    // println!("Pop back: {:?}", list.pop_back().unwrap().1.x);
    // println!("Pop back: {:?}", list.pop_back().unwrap().1.x);
}