use std::fmt;
use crate::function::Function;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Closure { 
    pub function: Function,
    // variables
}

impl Closure {
    pub fn new(function: Function) -> Self {
        Closure { function }
    }
}

impl fmt::Display for Closure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.function)
    }
}