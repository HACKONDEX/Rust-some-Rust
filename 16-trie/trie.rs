#![forbid(unsafe_code)]
use crate::trie_key::ToKeyIter;
use std::{
    collections::HashMap,
    ops::Index,
};

struct TrieNode<Key, Val>
where
    Key: ToKeyIter,
    Val: Copy,
{
    children: HashMap<Key::Item, TrieNode<Key, Val>>,
    terminals_count: usize,
    terminal: bool,
    value: Option<Val>,
}

impl<Key, Val> TrieNode<Key, Val>
where
    Key: ToKeyIter,
    Val: Copy,
{
    pub fn new() -> Self {
        TrieNode {
            children: HashMap::new(),
            terminals_count: 0,
            terminal: false,
            value: None,
        }
    }

    pub fn has_child(&self, key: &Key::Item) -> bool {
        self.children.contains_key(key)
    }

    pub fn get_mut_child(&mut self, key: &Key::Item) -> Option<&mut Self> {
        self.children.get_mut(key)
    }

    pub fn get_child(&self, key: &Key::Item) -> Option<&Self> {
        self.children.get(key)
    }

    pub fn insert_child(&mut self, key: &Key::Item) {
        self.children.insert(key.clone(), TrieNode::new());
    }

    pub fn is_terminal(&self) -> bool {
        self.terminal
    }

    pub fn remove_value(&mut self) -> Option<Val> {
        if !self.terminal {
            return None;
        }
        self.terminal = false;
        let old_value = self.value;
        self.value = None;
        old_value
    }

    pub fn set_value(&mut self, new_value: Val) {
        self.value = Some(new_value);
        self.terminal = true;
    }

    pub fn get_value(&self) -> Option<&Val> {
        self.value.as_ref()
    }

    pub fn get_mut_value(&mut self) -> Option<&mut Val> {
        self.value.as_mut()
    }

    pub fn get_terminals_count(&self) -> usize {
        self.terminals_count
    }

    pub fn update_terminals_count(&mut self, plus: bool) {
        if plus {
            self.terminals_count += 1;
        } else {
            self.terminals_count -= 1;
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Trie<Key, Val>
where
    Key: ToKeyIter,
    Val: Copy,
{
    root: Option<TrieNode<Key, Val>>,
}

impl<Key, Val> Trie<Key, Val>
where
    Key: ToKeyIter,
    Val: Copy,
{
    pub fn new() -> Self {
        Self {
            root: Some(TrieNode::new()),
        }
    }

    pub fn len(&self) -> usize {
        self.root.as_ref().unwrap().get_terminals_count()
    }

    pub fn is_empty(&self) -> bool {
        self.root.as_ref().unwrap().get_terminals_count() == 0
    }

    fn get_node<Q: ?Sized>(&self, key: &Q) -> Option<&TrieNode<Key, Val>>
    where
        Q: ToKeyIter<Item = Key::Item>,
    {
        let mut iterator = key.key_iter();
        let mut current_node = self.root.as_ref().unwrap();
        'outer: loop {
            match iterator.next() {
                None => {
                    break 'outer;
                }
                Some(symbol) => {
                    if !current_node.has_child(&symbol) {
                        return None;
                    }
                    current_node = current_node.get_child(&symbol).unwrap();
                }
            }
        }
        Some(current_node)
    }

    fn get_mut_node<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut TrieNode<Key, Val>>
    where
        Q: ToKeyIter<Item = Key::Item>,
    {
        let mut iterator = key.key_iter();
        let mut current_node = self.root.as_mut().unwrap();
        'outer: loop {
            match iterator.next() {
                None => {
                    break 'outer;
                }
                Some(symbol) => {
                    if !current_node.has_child(&symbol) {
                        return None;
                    }
                    current_node = current_node.get_mut_child(&symbol).unwrap();
                }
            }
        }
        Some(current_node)
    }

    fn update_terminals_count<Q: ?Sized>(&mut self, key: &Q, plus: bool)
    where
        Q: ToKeyIter<Item = Key::Item>,
    {
        let mut iterator = key.key_iter();
        let mut current_node = self.root.as_mut().unwrap();
        'outer: loop {
            current_node.update_terminals_count(plus);
            match iterator.next() {
                None => {
                    break 'outer;
                }
                Some(symbol) => {
                    if !current_node.has_child(&symbol) {
                        return;
                    }
                    current_node = current_node.get_mut_child(&symbol).unwrap();
                }
            }
        }
    }

    pub fn insert<Q: ?Sized>(&mut self, key: &Q, value: Val) -> Option<Val>
    where
        Q: ToKeyIter<Item = Key::Item>,
    {
        let mut iterator = key.key_iter();
        let mut current_node = self.root.as_mut().unwrap();
        'outer: loop {
            let ch = iterator.next();
            match ch {
                None => {
                    break 'outer;
                }
                Some(item) => {
                    if !current_node.has_child(&item) {
                        current_node.insert_child(&item);
                    }
                    current_node = current_node.get_mut_child(&item).unwrap();
                }
            }
        }
        let old_value = current_node.remove_value();
        current_node.set_value(value);
        if old_value.is_none() {
            self.update_terminals_count(key, true);
        }
        old_value
    }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&Val>
    where
        Q: ToKeyIter<Item = Key::Item>,
    {
        let destination = self.get_node(key);
        match destination {
            None => None,
            Some(node) => node.get_value(),
        }
    }

    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut Val>
    where
        Q: ToKeyIter<Item = Key::Item>,
    {
        let destination = self.get_mut_node(key);
        match destination {
            None => None,
            Some(node) => node.get_mut_value(),
        }
    }

    pub fn contains<Q: ?Sized>(&self, key: &Q) -> bool
    where
        Q: ToKeyIter<Item = Key::Item>,
    {
        let destination = self.get_node(key);
        match destination {
            None => false,
            Some(node) => node.is_terminal(),
        }
    }

    pub fn starts_with<Q: ?Sized>(&self, key: &Q) -> bool
    where
        Q: ToKeyIter<Item = Key::Item>,
    {
        let destination = self.get_node(key);
        if destination.is_none() {
            return false;
        }
        destination.unwrap().get_terminals_count() > 0
    }

    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<Val>
    where
        Q: ToKeyIter<Item = Key::Item>,
    {
        let destination = self.get_mut_node(key);
        let mut ans: Option<Val> = None;
        match destination {
            None => {}
            Some(node) => {
                if node.is_terminal() {
                    ans = node.remove_value();
                    self.update_terminals_count(key, false);
                }
            }
        };
        ans
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<Key, Val, Q: ?Sized> Index<&Q> for Trie<Key, Val>
where
    Key: ToKeyIter,
    Val: Copy,
    Q: ToKeyIter<Item = Key::Item>,
{
    type Output = Val;
    fn index(&self, index: &Q) -> &Val {
        match self.get_node(index) {
            None => panic!("No node for key"),
            Some(node) => {
                if node.is_terminal() {
                    return node.value.as_ref().unwrap();
                }
                panic!("No value for key");
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////

impl<Key, Val> Default for Trie<Key, Val>
where
    Key: ToKeyIter,
    Val: Copy,
{
    fn default() -> Self {
        Self::new()
    }
}
