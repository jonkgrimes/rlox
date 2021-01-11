use std::rc::Rc;

use crate::core::Value; 
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct UpvalueRef {
    local: bool,
    location: Rc<Value>
}

impl UpvalueRef {
    pub fn local(&self) -> bool {
        self.local
    }

    pub fn location_ref(&self) -> Rc<Value> {
        self.location.clone()
    }

}