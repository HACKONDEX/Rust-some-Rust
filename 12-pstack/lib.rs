#![forbid(unsafe_code)]

use std::rc::Rc;

pub struct Node<T> {
    value: Option<T>,
    size: usize,
    previous: Option<Rc<Node<T>>>,
}

impl<T> Node<T> {
    pub fn default() -> Self {
        Self {
            value: None,
            size: 0,
            previous: None,
        }
    }

    pub fn new(value: T) -> Self {
        Self {
            value: Some(value),
            size: 1,
            previous: None,
        }
    }
}

pub struct PRef<T> {
    rc_node: Rc<Node<T>>,
}

impl<T> std::ops::Deref for PRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.rc_node.as_ref().value.as_ref().unwrap()
    }
}

impl<T> PRef<T> {
    pub fn default() -> Self {
        Self {
            rc_node: Rc::new(Node::<T>::default()),
        }
    }

    pub fn new(rc_node: Rc<Node<T>>) -> Self {
        Self { rc_node }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct PStack<T> {
    head: PRef<T>,
}

impl<T> Default for PStack<T> {
    fn default() -> Self {
        Self {
            head: PRef::<T>::default(),
        }
    }
}

impl<T> PStack<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_p_ref(other_head: &PRef<T>) -> Self {
        let mut new_pstack = Self::default();
        new_pstack.head.rc_node = other_head.rc_node.clone();
        new_pstack
    }

    pub fn push(&self, value: T) -> Self {
        let mut new_pstack = Self::from_p_ref(&self.head);
        let old_size = new_pstack.head.rc_node.size;
        let mut new_node = Node::<T>::new(value);
        new_node.size = old_size + 1;
        new_node.previous = Some(new_pstack.head.rc_node.clone());
        new_pstack.head.rc_node = Rc::new(new_node);
        new_pstack
    }

    pub fn pop(&self) -> Option<(PRef<T>, Self)> {
        self.head.rc_node.previous.as_ref()?;
        let mut stack = Self::from_p_ref(&self.head);
        let top = PRef::<T>::new(stack.head.rc_node.clone());
        stack.head = PRef::new(top.rc_node.previous.as_ref().unwrap().clone());
        Some((top, stack))
    }

    pub fn len(&self) -> usize {
        self.head.rc_node.size
    }

    pub fn is_empty(&self) -> bool {
        self.head.rc_node.size == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = PRef<T>> {
        PStackIterator::<T>::new(self)
    }
}

pub struct PStackIterator<T> {
    head: PRef<T>,
}

impl<T> PStackIterator<T> {
    pub fn new(stack: &PStack<T>) -> Self {
        Self {
            head: PRef::<T>::new(stack.head.rc_node.clone()),
        }
    }
}

impl<T> Iterator for PStackIterator<T> {
    type Item = PRef<T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.head.rc_node.previous.as_ref()?;
        let top = self.head.rc_node.clone();
        self.head = PRef::<T>::new(top.previous.as_ref().unwrap().clone());
        Some(PRef::<T>::new(top))
    }
}

impl<T> Clone for PStack<T> {
    fn clone(&self) -> Self {
        Self::from_p_ref(&self.head)
    }
}
