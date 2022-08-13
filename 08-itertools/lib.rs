#![forbid(unsafe_code)]

use ::std::cell::RefCell;
use ::std::rc::Rc;
use std::{cmp::Ordering, collections::VecDeque};

#[derive(Default)]
pub struct LazyCycle<I>
where
    I: Iterator,
    I::Item: Clone,
{
    buffer: Vec<I::Item>,
    iter: I,
    is_exhausted: bool,
    id: usize,
}

impl<I> LazyCycle<I>
where
    I: Iterator,
    I::Item: Clone,
{
    pub fn new(iter_: I) -> Self {
        Self {
            buffer: Vec::new(),
            iter: iter_,
            is_exhausted: false,
            id: 0,
        }
    }
}

impl<I> Iterator for LazyCycle<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<I::Item> {
        if self.is_exhausted {
            if self.buffer.is_empty() {
                return None;
            }
            match self.id.cmp(&self.buffer.len()) {
                Ordering::Less => {
                    self.id += 1;
                    Some(self.buffer[self.id - 1].clone())
                }
                Ordering::Equal => {
                    self.id = 1;
                    Some(self.buffer[0].clone())
                }
                Ordering::Greater => None,
            }
        } else {
            match self.iter.next() {
                Some(t) => {
                    self.buffer.push(t.clone());
                    Some(t)
                }
                None => {
                    self.is_exhausted = true;
                    self.next()
                }
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[allow(clippy::needless_collect)]
pub struct Extract<I: Iterator> {
    buffer: VecDeque<I::Item>,
}

impl<I> Extract<I>
where
    I: Iterator,
{
    pub fn new(buffer_: VecDeque<I::Item>) -> Self {
        Self { buffer: buffer_ }
    }
}

impl<I> Iterator for Extract<I>
where
    I: Iterator,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<I::Item> {
        if self.buffer.is_empty() {
            return None;
        }
        self.buffer.pop_front()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Tee<I>
where
    I: Iterator,
    I::Item: Clone,
{
    buffer: Rc<RefCell<VecDeque<I::Item>>>,
    rc_iter: Rc<RefCell<I>>,
    exhausted_id: Rc<RefCell<usize>>,
    idx: usize,
    is_empty: bool,
}

impl<I> Tee<I>
where
    I: Iterator,
    I::Item: Clone,
{
    pub fn new(
        iter: Rc<RefCell<I>>,
        buf: Rc<RefCell<VecDeque<I::Item>>>,
        flag: Rc<RefCell<usize>>,
    ) -> Self {
        Self {
            buffer: buf,
            rc_iter: iter,
            exhausted_id: flag,
            idx: 0,
            is_empty: false,
        }
    }
}

impl<I> Iterator for Tee<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        if self.is_empty {
            return None;
        }
        if *self.exhausted_id.as_ref().borrow() > self.idx {
            if self.buffer.as_ref().borrow().is_empty() {
                self.is_empty = true;
                return None;
            }
            self.idx += 1;
            return self.buffer.as_ref().borrow_mut().pop_front();
        } else {
            *self.exhausted_id.as_ref().borrow_mut() += 1;
            match self.rc_iter.as_ref().borrow_mut().next() {
                Some(t) => {
                    self.buffer.as_ref().borrow_mut().push_back(t.clone());
                    self.idx += 1;
                    Some(t)
                }
                None => {
                    self.is_empty = true;
                    None
                }
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct GroupBy<I, F, V>
where
    I: Iterator,
    F: FnMut(&I::Item) -> V,
    V: Eq,
{
    buffer: VecDeque<(V, Vec<I::Item>)>,
    _f: F,
}

impl<I, F, V> GroupBy<I, F, V>
where
    I: Iterator,
    F: FnMut(&I::Item) -> V,
    V: Eq,
{
    pub fn new(vec: VecDeque<(V, Vec<I::Item>)>, f_: F) -> Self {
        Self {
            buffer: vec,
            _f: f_,
        }
    }
}

impl<I, F, V> Iterator for GroupBy<I, F, V>
where
    I: Iterator,
    F: FnMut(&I::Item) -> V,
    V: Eq,
{
    type Item = (V, Vec<I::Item>);
    fn next(&mut self) -> Option<(V, Vec<I::Item>)> {
        if self.buffer.is_empty() {
            return None;
        }

        self.buffer.pop_front()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub trait ExtendedIterator: Iterator {
    fn lazy_cycle(self) -> LazyCycle<Self>
    where
        Self: Sized,
        Self::Item: Clone,
    {
        LazyCycle::new(self)
    }

    fn extract(mut self, index: usize) -> (Option<Self::Item>, Extract<Self>)
    where
        Self: Sized,
    {
        let mut buf = VecDeque::new();
        let mut item = self.next();
        let mut id: usize = 0;
        let mut n_th = None;
        while item.is_some() {
            if id != index {
                buf.push_back(item.unwrap());
            } else {
                n_th = item;
            }
            id += 1;
            item = self.next();
        }
        (n_th, Extract::new(buf))
    }

    fn tee(self) -> (Tee<Self>, Tee<Self>)
    where
        Self: Sized,
        Self::Item: Clone,
    {
        let rc_iter = Rc::from(RefCell::from(self));
        let buffer = Rc::new(RefCell::from(VecDeque::new()));
        let shared_flag = Rc::new(RefCell::from(0));
        (
            Tee::new(rc_iter.clone(), buffer.clone(), shared_flag.clone()),
            Tee::new(rc_iter, buffer, shared_flag),
        )
    }

    fn group_by<F, V>(self, mut func: F) -> GroupBy<Self, F, V>
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> V,
        V: Eq,
    {
        let mut vec: VecDeque<(V, Vec<Self::Item>)> = VecDeque::new();
        for item in self {
            let func_res: V = func(&item);
            if !vec.is_empty() && func_res == vec.back().unwrap().0 {
                vec.back_mut().unwrap().1.push(item);
            } else {
                vec.push_back((func_res, vec![item]));
            }
        }
        GroupBy::new(vec, func)
    }
}

impl<I: Iterator> ExtendedIterator for I {}
