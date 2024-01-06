use std::rc::Rc;
use std::cell::{Ref, RefCell};
use std::collections::HashMap;

pub struct LRUCache<T> {
    size: usize,
    capacity: usize,
    head: Link<T>,
    tail: Link<T>,
    map: HashMap<String, Link<T>>,
}

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

struct Node<T> {
    key: String,
    elem: T,
    next: Link<T>,
    prev: Link<T>,
}

#[derive(Clone)]
struct NodeEntry<T> {
    key: String,
    elem: T,
}

impl<T: Clone> Node<T> {
    fn new(entry: NodeEntry<T>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            key: entry.key,
            elem: entry.elem,
            prev: None,
            next: None,
        }))
    }
}

impl<T: Clone> LRUCache<T> {
    fn delete_node(&mut self, node: &mut Link<T>) -> NodeEntry<T> {
        let node = node.take().unwrap();
        let pre_node = node.borrow_mut().prev.take();
        let next_node = node.borrow_mut().next.take();
        let entry = NodeEntry {
            key: node.borrow().key.clone(),
            elem: node.borrow().elem.clone(),
        };
        self.map.remove(&entry.key);
        self.size -= 1;
        if pre_node.is_none() && next_node.is_none() {
            self.head.take();
            self.tail.take();
            return entry;
        }
        if pre_node.is_none() {
            let next_node = next_node.unwrap();
            next_node.borrow_mut().prev.take();
            self.head = Some(Rc::clone(&next_node));
            return entry;
        }
        if next_node.is_none() {
            let pre_node = pre_node.unwrap();
            pre_node.borrow_mut().next.take();
            self.tail = Some(Rc::clone(&pre_node));
            return entry;
        }
        let pre_node = pre_node.unwrap();
        let next_node = next_node.unwrap();
        pre_node.borrow_mut().next = Some(Rc::clone(&next_node));
        next_node.borrow_mut().prev = Some(Rc::clone(&pre_node));
        entry
    }

    fn push_front(&mut self, entry: NodeEntry<T>) {
        let new_head = Node::new(entry.clone());
        match self.head.take() {
            Some(old_head) => {
                old_head.borrow_mut().prev = Some(new_head.clone());
                new_head.borrow_mut().next = Some(old_head);
                self.head = Some(new_head.clone());
            }
            None => {
                self.tail = Some(new_head.clone());
                self.head = Some(new_head.clone());
            }
        }
        self.size += 1;
        self.map.insert(entry.key,Some(new_head));
    }

    fn pop_back(&mut self) -> Option<T> {
        self.tail.take().map(|old_tail| {
            match old_tail.borrow_mut().prev.take() {
                Some(new_tail) => {
                    new_tail.borrow_mut().next.take();
                    self.tail = Some(new_tail);
                }
                None => {
                    self.head.take();
                }
            }
            self.size -= 1;
            self.map.remove(&old_tail.borrow().key);
            Rc::try_unwrap(old_tail).ok().unwrap().into_inner().elem
        })
    }
}

impl <T: Clone> LRUCache<T> {
    pub fn new(capacity: usize) -> Self {
        LRUCache {
            capacity,
            size: 0,
            head: None, 
            tail: None,
            map: HashMap::with_capacity(capacity),
        }
    }

    pub fn get_size(&self) -> usize {
        self.size
    }

    pub fn contains_key(&self, key: String) -> bool {
        self.map.contains_key(&key)
    }

    pub fn get(&mut self, key: String) -> Option<Ref<T>> {
        if !self.map.contains_key(&key) {
            return None;
        }
        let node = self.map.get(&key).unwrap();
        let node = node.as_ref().unwrap();
        let mut node = Some(Rc::clone(node));
        let entry = self.delete_node(&mut node);
        self.push_front(entry);
        let node = self.map[&key].as_ref().unwrap();
        Some(Ref::map(node.borrow(), |node| &node.elem))
    }

    pub fn put(&mut self, key: String, value: T) {
        if self.map.contains_key(&key) {
            let node = self.map.get(&key).unwrap();
            let node = node.as_ref().unwrap();
            let mut node = Some(Rc::clone(node));
            let mut entry = self.delete_node(&mut node);
            entry.elem = value;
            self.push_front(entry);
            return;
        }
        if self.size == self.capacity {
            let _ = self.pop_back();
        }
        let entry = NodeEntry {
            key,
            elem: value,
        };
        self.push_front(entry);
    }

    pub fn remove(&mut self, key: String) {
        if self.map.contains_key(&key) {
            let node = self.map.get(&key).unwrap();
            let node = node.as_ref().unwrap();
            let mut node = Some(Rc::clone(node));
            self.delete_node(&mut node);
        }
    }
}