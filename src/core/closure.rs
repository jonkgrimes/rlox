use crate::core::Function;
use crate::core::UpvalueRef;
use std::fmt;

use super::function;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Closure {
    pub function: Function,
    pub upvalues: Vec<UpvalueRef>,
    pub upvalue_count: usize
}

impl Closure {
    pub fn new(function: Function) -> Self {
        let upvalues: Vec<UpvalueRef> = Vec::new();
        let upvalue_count = function.upvalue_count;
        Closure { function, upvalues, upvalue_count }
    }
}

impl fmt::Display for Closure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.function)
    }
}
