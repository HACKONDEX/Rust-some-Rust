#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug)]
pub struct LRUCache<K, V> {
    capacity: usize,
    size: usize,
    min_prior: u32,
    max_prior: u32,
    key_prior_map: HashMap<K, u32>,
    prior_value_map: HashMap<u32, (K, V)>,
}

impl<K: Clone + Hash + Ord, V> LRUCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        if capacity == 0 {
            panic!()
        }
        Self {
            capacity,
            size: 0,
            min_prior: 0,
            max_prior: 0,
            key_prior_map: HashMap::new(),
            prior_value_map: HashMap::new(),
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        if self.key_prior_map.contains_key(key) {
            self.get_and_update_prior(key)
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.key_prior_map.contains_key(&key) {
            self.get_old_and_insert(key, value)
        } else {
            self.insert_new(key, value);
            None
        }
    }

    pub fn get_new_prior(&mut self) -> u32 {
        let new_prior = self.max_prior;
        self.max_prior += 1;
        new_prior
    }

    pub fn get_and_update_prior(&mut self, key: &K) -> Option<&V> {
        let prior = self.key_prior_map.remove(key).unwrap();

        if self.prior_value_map.contains_key(&prior) {
            let new_prior = self.update_priors(prior);
            Some(&self.prior_value_map.get(&new_prior).unwrap().1)
        } else {
            None
        }
    }

    pub fn update_priors(&mut self, prior: u32) -> u32 {
        let (key, value) = self.prior_value_map.remove(&prior).unwrap();

        if prior == self.min_prior {
            self.min_prior += 1;
        }

        self.simple_insert(key, value)
    }

    pub fn simple_insert(&mut self, key: K, value: V) -> u32 {
        let new_prior = self.get_new_prior();
        self.key_prior_map.insert(key.clone(), new_prior);
        self.prior_value_map.insert(new_prior, (key, value));
        new_prior
    }

    pub fn remove_least_recent(&mut self) {
        while !self.prior_value_map.contains_key(&self.min_prior) {
            self.min_prior += 1;
        }
        let key = self.prior_value_map.remove(&self.min_prior).unwrap().0;
        self.key_prior_map.remove(&key);
        self.min_prior += 1;
    }

    pub fn insert_new(&mut self, key: K, value: V) {
        if self.size < self.capacity {
            self.size += 1;
        } else {
            self.remove_least_recent();
        }
        self.simple_insert(key, value);
    }

    pub fn get_old_and_insert(&mut self, key: K, value: V) -> Option<V> {
        let prior = self.key_prior_map.remove(&key).unwrap();
        let old_value = self.prior_value_map.remove(&prior).unwrap().1;
        self.simple_insert(key, value);

        Some(old_value)
    }
}
