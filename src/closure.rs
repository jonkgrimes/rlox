use crate::{function::Function, upvalue::Upvalue};
use std::fmt;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Closure {
    pub function: Function,
    pub upvalues: Vec<Upvalue>,
}

impl Closure {
    pub fn new(function: Function) -> Self {
        let upvalues: Vec<Upvalue> = Vec::new();
        Closure { function, upvalues }
    }
}

impl fmt::Display for Closure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.function)
    }
}
