#![forbid(unsafe_code)]

pub struct MinStack<T> {
    buffer: Vec<(T, T)>,
}

impl<T: Clone + Ord> Default for MinStack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Ord> MinStack<T> {
    pub fn new() -> Self {
        Self {
            buffer: Vec::<(T, T)>::new(),
        }
    }

    pub fn push(&mut self, value: T) {
        match self.buffer.len() {
            0 => self.buffer.push((value.to_owned(), value)),
            n => self.buffer.push(if self.buffer[n - 1].1 < value {
                (value, self.buffer[n - 1].1.to_owned())
            } else {
                (value.to_owned(), value)
            }),
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        let result = self.buffer.pop();
        match result {
            Option::<(T, T)>::Some(pair) => Some(pair.0),
            _ => Option::<T>::None,
        }
    }

    pub fn min(&self) -> Option<&T> {
        match self.buffer.len() {
            0 => None,
            n => Some(&self.buffer[n - 1].1),
        }
    }

    pub fn top(&self) -> Option<&T> {
        match self.buffer.len() {
            0 => None,
            n => Some(&self.buffer[n - 1].0),
        }
    }

    pub fn bottom(&self) -> Option<&T> {
        if self.buffer.is_empty() {
            None
        } else {
            Some(&self.buffer.first().unwrap().0)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}

pub struct MinQueue<T> {
    front: MinStack<T>,
    back: MinStack<T>,
}

impl<T: Clone + Ord> Default for MinQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Ord> MinQueue<T> {
    pub fn new() -> Self {
        Self {
            front: MinStack::<T>::new(),
            back: MinStack::<T>::new(),
        }
    }

    pub fn push(&mut self, val: T) {
        self.front.push(val);
    }

    pub fn pop(&mut self) -> Option<T> {
        match (self.front.is_empty(), self.back.is_empty()) {
            (true, true) => None,
            (_, false) => self.back.pop(),
            (false, true) => self.pull_and_pop(),
        }
    }

    pub fn front(&self) -> Option<&T> {
        match (self.front.is_empty(), self.back.is_empty()) {
            (_, false) => self.back.top(),
            (_, true) => self.front.bottom(),
        }
    }

    pub fn min(&self) -> Option<&T> {
        match (self.front.is_empty(), self.back.is_empty()) {
            (true, true) => None,
            (false, true) => self.front.min(),
            (true, false) => self.back.min(),
            (false, false) => Some(std::cmp::min(
                self.front.min().unwrap(),
                self.back.min().unwrap(),
            )),
        }
    }

    pub fn len(&self) -> usize {
        self.front.len() + self.back.len()
    }

    pub fn is_empty(&self) -> bool {
        self.front.is_empty() && self.back.is_empty()
    }

    pub fn pull_to_back(&mut self) {
        while !self.front.is_empty() {
            self.back.push(self.front.pop().unwrap());
        }
    }

    pub fn pull_and_pop(&mut self) -> Option<T> {
        self.pull_to_back();
        self.back.pop()
    }
}
