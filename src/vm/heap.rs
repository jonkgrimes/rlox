use slotmap::{DefaultKey, SlotMap};
use crate::core::Object;

pub type ObjectId = DefaultKey;

pub struct Heap {
    data: SlotMap<ObjectId, Object>,
    object_id_counter: usize
}

impl Heap {
    pub fn new() -> Self {
        let data = SlotMap::new();
        let object_id_counter = 1;
        Heap { data, object_id_counter }
    }

    pub fn add_value(&mut self, value: Object) -> ObjectId {
        let id = self.data.insert(value);
        self.object_id_counter += 1;
        id
    }

    pub fn get(&self, object_id: &ObjectId) -> Option<&Object> {
        self.data.get(*object_id)
    }

    pub fn get_mut(&mut self, object_id: &ObjectId) -> Option<&mut Object> {
        self.data.get_mut(*object_id)
    }
}