use std::collections::HashMap;
use crate::core::Value;

pub type ObjectId = usize;

pub struct Heap {
    data: HashMap<ObjectId, Value>,
    object_id_counter: usize
}

impl Heap {
    pub fn new() -> Self {
        let data = HashMap::new();
        let object_id_counter = 1;
        Heap { data, object_id_counter }
    }

    pub fn insert(&mut self, value: Value) -> bool {
        let key_existed = self.data.insert(self.object_id_counter, value);
        self.object_id_counter += 1;
        key_existed.is_some()
    }

    pub fn get(&self, object_id: &ObjectId) -> Option<&Value> {
        self.data.get(object_id)
    }

    pub fn get_mut(&mut self, object_id: &ObjectId) -> Option<&mut Value> {
        self.data.get_mut(object_id)
    }
}