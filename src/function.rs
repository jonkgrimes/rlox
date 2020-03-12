use std::cmp::Ordering;
use std::fmt;

use crate::chunk::Chunk;

#[derive(Debug, Clone)]
pub struct Function {
    name: String,
    pub arity: usize,
    pub chunk: Chunk,
}

#[derive(Debug, Clone)]
pub enum FunctionType {
    Script,
    Function,
}

impl Function {
    pub fn new(name: &str) -> Function {
        Function {
            name: String::from(name),
            arity: 0,
            chunk: Chunk::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn disassemble(&self) {
        self.chunk.disassemble(&self.name)
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        println!("{:?}", self.chunk);
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
