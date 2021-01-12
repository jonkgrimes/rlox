use std::rc::Rc;
use std::fmt;

use crate::core::Value; 
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct UpvalueRef {
    local: bool,
    location: *mut Value
}

impl UpvalueRef {
    pub fn local(&self) -> bool {
        self.local
    }
}

impl fmt::Display for UpvalueRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.local {
          write!(f, "Local value")
        } else {
          write!(f, "Upvalue")
        }
    }
}
