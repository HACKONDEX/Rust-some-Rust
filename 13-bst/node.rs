#![forbid(unsafe_code)]

pub struct Node<K, V> {
    key: Option<K>,
    value: Option<V>,
    height: usize,
    count: usize,
    left: Option<Box<Node<K, V>>>,
    right: Option<Box<Node<K, V>>>,
}

impl<K: Ord, V> Node<K, V> {
    pub fn new(key: K, value: V) -> Self {
        Self {
            key: Some(key),
            value: Some(value),
            height: 1,
            count: 1,
            left: None,
            right: None,
        }
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn ref_key(&self) -> &K {
        self.key.as_ref().unwrap()
    }

    pub fn ref_value(&self) -> &V {
        self.value.as_ref().unwrap()
    }

    pub fn left_son_node(&self) -> Option<&Node<K, V>> {
        self.left.as_deref()
    }

    pub fn right_son_node(&self) -> Option<&Node<K, V>> {
        self.right.as_deref()
    }

    pub fn take_left_son(&mut self) -> Option<Box<Node<K, V>>> {
        self.left.take()
    }

    pub fn take_right_son(&mut self) -> Option<Box<Node<K, V>>> {
        self.right.take()
    }

    pub fn ref_key_value(&self) -> (&K, &V) {
        (self.key.as_ref().unwrap(), self.value.as_ref().unwrap())
    }

    pub fn set_left_son(&mut self, node: Option<Box<Node<K, V>>>) {
        self.left = node;
    }

    pub fn set_right_son(&mut self, node: Option<Box<Node<K, V>>>) {
        self.right = node;
    }

    pub fn take_value(&mut self) -> Option<V> {
        self.value.take()
    }

    pub fn set_value(&mut self, value: V) {
        self.value = Some(value);
    }

    pub fn take_key(&mut self) -> Option<K> {
        self.key.take()
    }

    pub fn fix_stats(&mut self) {
        self.count = self.right.as_deref().map_or(0, |x| x.count)
            + self.left.as_deref().map_or(0, |x| x.count)
            + 1;
        self.height = std::cmp::max(
            self.right.as_deref().map_or(0, |x| x.height),
            self.left.as_deref().map_or(0, |x| x.height),
        ) + 1;
    }

    pub fn get_difference(&self) -> i32 {
        self.right.as_deref().map_or(0_i32, |x| x.height as i32)
            - self.left.as_deref().map_or(0_i32, |x| x.height as i32)
    }
}
