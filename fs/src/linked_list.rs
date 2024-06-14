use alloc::boxed::Box;
use alloc::collections::BTreeMap;

pub struct Node<T> {
    value: Option<(isize, T)>,
    prev: Option<*mut Node<T>>,
    next: Option<*mut Node<T>>,
}

pub struct LinkedListWithMap<T> {
    head: Option<*mut Node<T>>,
    tail: Option<*mut Node<T>>,
    map: BTreeMap<isize, Box<Node<T>>>,
    size: usize,
}

impl<T> LinkedListWithMap<T> {
    pub fn new() -> Self {
        let mut map = BTreeMap::new();
        let mut head = Box::new(Node { value: None, prev: None, next: None });
        let mut tail = Box::new(Node { value: None, prev: None, next: None });
        unsafe {
            (*head).next = Some(tail.as_mut() as *mut Node<T>);
            (*tail).prev = Some(head.as_mut() as *mut Node<T>);
        }
        let head_ptr = Some(head.as_mut() as *mut Node<T>);
        let tail_ptr = Some(tail.as_mut() as *mut Node<T>);
        map.insert(-1, head);
        map.insert(-2, tail);
        LinkedListWithMap { head: head_ptr, tail: tail_ptr, map, size: 0 }
    }

    pub fn push_front(&mut self, key: isize, value: T) {
        let mut new_node = Box::new(Node { value: Some((key, value)), prev: None, next: None });
        unsafe {
            let prev = self.head.map(|p| p).unwrap();
            let next = (*prev).next.map(|p| p).unwrap();
            (*next).prev = Some(new_node.as_mut() as *mut Node<T>);
            (*prev).next = Some(new_node.as_mut() as *mut Node<T>);
            (*new_node).prev = Some(prev);
            (*new_node).next = Some(next);
        }
        self.map.insert(key, new_node);
        self.size += 1;
    }

    pub fn move_to_head(&mut self, key: isize) {
        unsafe {
            let mv_node = self.map.get(&key).unwrap().as_ref() as *const Node<T> as *mut Node<T>;
            let old_prev = (*mv_node).prev.map(|p| p).unwrap();
            let old_next = (*mv_node).next.map(|p| p).unwrap();
            (*old_next).prev = Some(old_prev);
            (*old_prev).next = Some(old_next);
            let prev = self.head.map(|p| p).unwrap();
            let next = (*prev).next.map(|p| p).unwrap();
            (*next).prev = Some(mv_node);
            (*prev).next = Some(mv_node);
            (*mv_node).prev = Some(prev);
            (*mv_node).next = Some(next);
        }
    }

    pub fn pop_back(&mut self) -> Option<(isize, T)> {
        if self.size == 0 {
            return None;
        }
        unsafe {
            let next = self.tail.map(|p| p).unwrap();
            let node = (*next).prev.map(|p| p).unwrap();
            let prev = (*node).prev.map(|p| p).unwrap();
            (*next).prev = Some(prev);
            (*prev).next = Some(next);
            let (key, value) = (*node).value.take().unwrap();
            self.map.remove(&key);
            self.size -= 1;
            return Some((key, value));
        }
    }

    pub fn find(&mut self, key: isize) -> Option<&(isize, T)> {
        unsafe {
            if let Some(node) = self.map.get(&key) {
                node.value.as_ref()
            } else {
                None
            }
        }
    }
}
