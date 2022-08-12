#![forbid(unsafe_code)]

use std::any::{Any, TypeId};
use std::collections::HashMap;

pub struct Context
where
    Self: Sized,
{
    map: HashMap<String, Box<dyn Any>>,
    set: HashMap<TypeId, Box<dyn Any>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            set: HashMap::new(),
        }
    }

    pub fn insert<Key: ToString, Val: Any>(&mut self, key: Key, obj: Val) {
        let key_string = key.to_string();
        self.map.insert(key_string, Box::new(obj));
    }

    pub fn get<Val: Any>(&self, key: &str) -> &Val {
        let key_string = key.to_string();
        match self.map.get(&key_string) {
            None => panic!("Not value for key"),
            Some(value) => {
                let casted = value.downcast_ref::<Val>();
                if casted.is_none() {
                    panic!("The underlying values type is not matching type in Generic parameter")
                }
                casted.unwrap()
            }
        }
    }

    pub fn insert_singletone<Val: Any>(&mut self, obj: Val) {
        let type_id = TypeId::of::<Val>();
        self.set.remove(&type_id);
        self.set.insert(type_id, Box::new(obj));
    }

    pub fn get_singletone<Val: Any>(&self) -> &Val {
        let type_id = TypeId::of::<Val>();
        let value = self.set.get(&type_id);
        match value {
            None => {
                panic!("There is no such value!")
            }
            Some(value) => {
                let value_val = value.downcast_ref::<Val>();
                if value_val.is_none() {
                    panic!("The underlying values type is not matching type in Generic parameter");
                }
                value_val.unwrap()
            }
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}
