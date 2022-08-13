#![forbid(unsafe_code)]
use std::{hash::Hash, str::Chars};

pub trait ToKeyIter {
    type Item: Clone + Hash + Eq;
    type KeyIter<'b>: Iterator<Item = Self::Item>
    where
        Self: 'b;
    fn key_iter(&self) -> Self::KeyIter<'_>;
}

impl ToKeyIter for str {
    type Item = char;
    type KeyIter<'a> = Chars<'a>;

    fn key_iter<'a>(&'a self) -> Chars<'a> {
        let x: Chars<'a> = self.chars();
        x
    }
}

impl ToKeyIter for String {
    type Item = char;
    type KeyIter<'a> = Chars<'a>;

    fn key_iter<'a>(&'a self) -> Chars<'a> {
        let x: Chars<'a> = self.chars();
        x
    }
}

////////////////////////////////////////////////////////////////////////////////

// Bonus

// pub trait FromKeyIter {
//     fn to_key(self) -> ???;
// }

// impl FromKeyIter for ???
// TODO: your code goes here.

////////////////////////////////////////////////////////////////////////////////

// Bonus

// pub trait TrieKey
// TODO: your code goes here.
