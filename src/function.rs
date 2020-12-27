use std::cmp::Ordering;
use std::fmt;

use crate::chunk::Chunk;
use crate::upvalue::Upvalue;

#[derive(Debug, Clone)]
pub struct Function {
    name: String,
    function_type: FunctionType,
    pub arity: usize,
    pub chunk: Chunk,
    pub upvalues: Vec<Upvalue>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionType {
    Script,
    Function,
    Native,
}

impl Function {
    pub fn new(name: &str, function_type: FunctionType) -> Function {
        Function {
            name: String::from(name),
            arity: 0,
            function_type,
            chunk: Chunk::new(),
            upvalues: Vec::new()
        }
    }

    pub fn disassemble(&self) {
        self.chunk.disassemble(&self.name)
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<fn {} airty: {}>", self.name, self.arity)
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Function) -> bool {
        self.name == other.name
    }
}

impl PartialOrd for Function {
    fn partial_cmp(&self, other: &Function) -> Option<Ordering> {
        self.name.partial_cmp(&other.name)
    }
}
