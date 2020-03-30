use crate::value::Value;
use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, Clone)]
pub struct NativeFunction {
    pub name: String,
    pub function: fn() -> Value,
}

impl PartialEq for NativeFunction {
    fn eq(&self, other: &NativeFunction) -> bool {
        self.name == other.name
    }
}

impl PartialOrd for NativeFunction {
    fn partial_cmp(&self, other: &NativeFunction) -> Option<Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl fmt::Display for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<fn {}>", self.name)
    }
}
