#![forbid(unsafe_code)]

use crate::node::Node;
use std::borrow::Borrow;
use std::cmp::Ordering;

// Took AVL C++ implementation from
// https://github.com/werticell/algo_cpp/blob/master/1_term/3_module/3_2.cpp
//
pub struct AVLTreeMap<K, V> {
    root: Option<Box<Node<K, V>>>,
}

impl<K: Ord, V> Default for AVLTreeMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

struct RemoveRet<K, V> {
    node: Option<Box<Node<K, V>>>,
    ret_k: Option<K>,
    ret_v: Option<V>,
}

impl<K, V> Default for RemoveRet<K, V> {
    fn default() -> Self {
        Self {
            node: None,
            ret_k: None,
            ret_v: None,
        }
    }
}

struct InsertRet<K, V> {
    node: Option<Box<Node<K, V>>>,
    old_value: Option<V>,
}

impl<K, V> InsertRet<K, V> {
    pub fn new(node: Option<Box<Node<K, V>>>, old_value: Option<V>) -> Self {
        Self { node, old_value }
    }
}

struct NodesPair<K, V> {
    tmp: Option<Box<Node<K, V>>>,
    correct_tree: Option<Box<Node<K, V>>>,
}

impl<K, V> NodesPair<K, V> {
    pub fn new(correct_tree: Option<Box<Node<K, V>>>, tmp: Option<Box<Node<K, V>>>) -> Self {
        Self { tmp, correct_tree }
    }
}

impl<K: Ord, V> AVLTreeMap<K, V> {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn len(&self) -> usize {
        Self::get_count(self.root.as_deref())
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        Some(self.get_key_value(key)?.1)
    }

    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        let mut current_node = self.root.as_deref()?;
        loop {
            match <K as Borrow<Q>>::borrow(current_node.ref_key()).cmp(key) {
                Ordering::Equal => {
                    return Some(current_node.ref_key_value());
                }
                Ordering::Greater => {
                    current_node = current_node.left_son_node()?;
                }
                Ordering::Less => {
                    current_node = current_node.right_son_node()?;
                }
            }
        }
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.get(key).is_some()
    }

    pub fn nth_key_value(&self, k: usize) -> Option<(&K, &V)> {
        let mut pos = k as i32;
        if Self::get_count(self.root.as_deref()) < k {
            return None;
        }
        let mut current_node = self.root.as_deref()?;
        while Self::get_count(current_node.left_son_node()) as i32 != pos {
            if pos > Self::get_count(current_node.left_son_node()) as i32 {
                pos -= (Self::get_count(current_node.left_son_node()) + 1) as i32;
                current_node = current_node.right_son_node()?;
            } else {
                current_node = current_node.left_son_node()?;
            }
        }
        return Some((current_node.ref_key(), current_node.ref_value()));
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let mut x = Self::insert_into_node(self.root.take(), key, value);
        self.root = x.node.take();
        x.old_value
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        Some(self.remove_entry(key)?.1)
    }

    pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        let mut x = Self::remove_from_node(self.root.take(), key);
        self.root = x.node.take();
        Some((x.ret_k.take()?, x.ret_v.take()?))
    }

    fn get_height(node: Option<&Node<K, V>>) -> usize {
        match node {
            None => 0_usize,
            Some(x) => x.height(),
        }
    }

    fn get_count(node: Option<&Node<K, V>>) -> usize {
        match node {
            None => 0_usize,
            Some(x) => x.count(),
        }
    }

    fn rotate_right(mut node: Box<Node<K, V>>) -> Box<Node<K, V>> {
        let mut left_son = node.take_left_son().unwrap();
        node.set_left_son(left_son.take_right_son());
        node.fix_stats();
        left_son.set_right_son(Some(node));
        left_son.fix_stats();
        left_son
    }

    fn rotate_left(mut node: Box<Node<K, V>>) -> Box<Node<K, V>> {
        let mut right_son = node.take_right_son().unwrap();
        node.set_right_son(right_son.take_left_son());
        node.fix_stats();
        right_son.set_left_son(Some(node));
        right_son.fix_stats();
        right_son
    }

    fn fix_balance(mut node: Box<Node<K, V>>) -> Box<Node<K, V>> {
        node.fix_stats();
        let diff = node.get_difference();
        if diff == -2 {
            let left_diff = node.left_son_node().unwrap().get_difference();
            if left_diff > 0 {
                let y = Self::rotate_left(node.take_left_son().unwrap());
                node.set_left_son(Some(y));
            }
            return Self::rotate_right(node);
        }

        if diff == 2 {
            let right_diff = node.right_son_node().unwrap().get_difference();
            if right_diff < 0 {
                let y = Self::rotate_right(node.take_right_son().unwrap());
                node.set_right_son(Some(y));
            }
            return Self::rotate_left(node);
        }
        node
    }

    fn insert_into_node(mut node: Option<Box<Node<K, V>>>, key: K, value: V) -> InsertRet<K, V> {
        if node.is_none() {
            return InsertRet::<K, V>::new(Some(Box::new(Node::<K, V>::new(key, value))), None);
        }
        let ret_value: Option<V>;
        let mut_node = node.as_mut().unwrap();
        match &key.cmp(mut_node.ref_key()) {
            Ordering::Equal => {
                ret_value = mut_node.take_value();
                mut_node.set_value(value);
            }
            Ordering::Greater => {
                let mut x = Self::insert_into_node(mut_node.take_right_son(), key, value);
                ret_value = x.old_value.take();
                mut_node.set_right_son(x.node);
            }
            Ordering::Less => {
                let mut x = Self::insert_into_node(mut_node.take_left_son(), key, value);
                ret_value = x.old_value.take();
                mut_node.set_left_son(x.node);
            }
        };
        InsertRet::<K, V>::new(Some(Self::fix_balance(node.unwrap())), ret_value)
    }

    fn swap_with_min(
        left_son: Option<Box<Node<K, V>>>,
        right_son: Option<Box<Node<K, V>>>,
    ) -> Option<Box<Node<K, V>>> {
        let mut pair = Self::recursive_take_min(right_son);
        pair.tmp
            .as_mut()
            .unwrap()
            .set_right_son(pair.correct_tree.take());
        pair.tmp.as_mut().unwrap().set_left_son(left_son);
        Some(Self::fix_balance(pair.tmp.unwrap()))
    }

    fn recursive_take_min(mut node: Option<Box<Node<K, V>>>) -> NodesPair<K, V> {
        let mut_node = node.as_mut().unwrap();
        if mut_node.left_son_node().is_none() {
            return NodesPair::<K, V>::new(mut_node.take_right_son(), node);
        }

        let mut pair = Self::recursive_take_min(mut_node.take_left_son());
        mut_node.set_left_son(pair.correct_tree.take());
        pair.correct_tree = Some(Self::fix_balance(node.unwrap()));
        pair
    }

    fn swap_with_max(
        left_son: Option<Box<Node<K, V>>>,
        right_son: Option<Box<Node<K, V>>>,
    ) -> Option<Box<Node<K, V>>> {
        let mut pair = Self::recursive_take_max(left_son);
        pair.tmp
            .as_mut()
            .unwrap()
            .set_left_son(pair.correct_tree.take());
        pair.tmp.as_mut().unwrap().set_right_son(right_son);
        Some(Self::fix_balance(pair.tmp.unwrap()))
    }

    fn recursive_take_max(mut node: Option<Box<Node<K, V>>>) -> NodesPair<K, V> {
        let mut_node = node.as_mut().unwrap();
        if mut_node.right_son_node().is_none() {
            return NodesPair::<K, V>::new(mut_node.take_left_son(), node);
        }

        let mut pair = Self::recursive_take_max(mut_node.take_right_son());
        mut_node.set_right_son(pair.correct_tree.take());
        pair.correct_tree = Some(Self::fix_balance(node.unwrap()));
        pair
    }

    fn remove_from_node<Q>(mut node: Option<Box<Node<K, V>>>, key: &Q) -> RemoveRet<K, V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        if node.is_none() {
            return RemoveRet::<K, V>::default();
        }
        let mut_node = node.as_mut().unwrap();
        let mut return_val = RemoveRet::<K, V>::default();
        match <K as Borrow<Q>>::borrow(mut_node.ref_key()).cmp(key) {
            Ordering::Equal => {
                // node->key == key
                // Than this node should be deleted
                let left_son = mut_node.take_left_son();
                let right_son = mut_node.take_right_son();
                return_val.ret_k = mut_node.take_key();
                return_val.ret_v = mut_node.take_value();
                if right_son.is_none() {
                    return_val.node = left_son;
                    return return_val;
                }
                if left_son.is_none() {
                    return_val.node = right_son;
                    return return_val;
                }

                // Both subtrees exits
                let differ = Self::get_height(right_son.as_deref()) as i32
                    - Self::get_height(left_son.as_deref()) as i32;
                if differ >= 0 {
                    return_val.node = Self::swap_with_min(left_son, right_son);
                } else {
                    return_val.node = Self::swap_with_max(left_son, right_son);
                }
                return return_val;
            }
            Ordering::Less => {
                // node->key < key
                return_val = Self::remove_from_node(mut_node.take_right_son(), key);
                mut_node.set_right_son(return_val.node.take());
            }
            Ordering::Greater => {
                // node->key > key
                return_val = Self::remove_from_node(mut_node.take_left_son(), key);
                mut_node.set_left_son(return_val.node.take());
            }
        }
        return_val.node = Some(Self::fix_balance(node.unwrap()));
        return_val
    }
}
