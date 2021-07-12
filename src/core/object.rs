use std::fmt;

use crate::core::{Closure, UpvalueRef, NativeFunction};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Object {
    String(String),
    NativeFunction(NativeFunction),
    Closure(Closure),
    Upvalue(UpvalueRef)
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::String(value) => write!(f, "{}", value),
            Object::NativeFunction(value) => write!(f, "{}", value),
            Object::Closure(object) => {
                write!(f, "{:?}", *object)
            },
            Object::Upvalue(value) => write!(f, "{}", value),
        }
    }
}

impl From<String> for Object {
    fn from(s: String) -> Object {
        Object::String(s)
    }
}