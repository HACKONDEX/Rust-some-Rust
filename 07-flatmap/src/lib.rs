#![forbid(unsafe_code)]

use std::cmp::Ordering;
use std::{borrow::Borrow, iter::FromIterator, ops::Index};
////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug, PartialEq, Eq)]
pub struct FlatMap<K, V>(Vec<(K, V)>);

impl<K: Ord, V> FlatMap<K, V> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    pub fn as_slice(&self) -> &[(K, V)] {
        self.0.as_slice()
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.0.is_empty() {
            self.0.push((key, value));
            return None;
        }

        let mut id = self.binary_search(&key);

        match key.cmp(&self.0[id].0) {
            Ordering::Greater => id += 1,
            Ordering::Less => {}
            Ordering::Equal => return self.replace(id, key, value),
        }
        self.0.insert(id, (key, value));
        None
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        if self.0.is_empty() {
            return None;
        }

        let id = self.binary_search(key);
        if self.0[id].0.borrow() == key {
            return Some(self.0[id].1.borrow());
        }
        None
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        if self.0.is_empty() {
            return None;
        }

        let id = self.binary_search(key);
        if self.0[id].0.borrow() == key {
            let element = self.0.remove(id);
            return Some(element.1);
        }
        None
    }

    pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        if self.0.is_empty() {
            return None;
        }

        let id = self.binary_search(key);
        if self.0[id].0.borrow() == key {
            let element = self.0.remove(id);
            return Some(element);
        }
        None
    }

    fn binary_search<Q>(&self, key: &Q) -> usize
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        let (mut left, mut right): (usize, usize) = (0, self.0.len());
        while left != right {
            let (mut old_left, mut old_right) = (left, right);
            let mid = (left + right) / 2;

            match key.cmp(self.0[mid].0.borrow()) {
                Ordering::Less => {
                    old_right = right;
                    right = mid;
                }
                Ordering::Greater => {
                    old_left = left;
                    left = mid;
                }
                Ordering::Equal => return mid,
            }

            if old_left == left && old_right == right {
                return left;
            }
        }
        left
    }

    #[inline]
    fn replace(&mut self, id: usize, key: K, value: V) -> Option<V> {
        let pair = self.0.remove(id);
        self.0.insert(id, (key, value));
        Some(pair.1)
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<K, Q, V> Index<&'_ Q> for FlatMap<K, V>
where
    K: Borrow<Q> + Ord,
    Q: Ord + ?Sized,
{
    type Output = V;
    fn index(&self, key: &Q) -> &V {
        self.get(key).expect("no entry fpund for key")
    }
}

impl<K, V> Extend<(K, V)> for FlatMap<K, V>
where
    K: Ord,
{
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        iter.into_iter().for_each(move |(key, value)| {
            self.insert(key, value);
        })
    }
}

impl<K, V> From<Vec<(K, V)>> for FlatMap<K, V>
where
    K: Ord,
{
    fn from(vec: Vec<(K, V)>) -> Self {
        let mut map = FlatMap::new();
        for (key, value) in vec {
            map.insert(key, value);
        }
        map
    }
}

impl<K, V> From<FlatMap<K, V>> for Vec<(K, V)>
where
    K: Ord,
{
    fn from(map: FlatMap<K, V>) -> Self {
        map.0
    }
}

impl<K, V> FromIterator<(K, V)> for FlatMap<K, V>
where
    K: Ord,
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut map = FlatMap::new();
        for (key, value) in iter {
            map.insert(key, value);
        }
        map
    }
}

impl<K, V> IntoIterator for FlatMap<K, V>
where
    K: Ord,
{
    type Item = (K, V);
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
