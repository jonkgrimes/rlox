use std::fmt;

use crate::chunk::Chunk;

pub struct Function {
  name: String,
  arity: u32,
  pub chunk: Chunk,
}

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
}

impl fmt::Display for Function {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<fn {}>", self.name)
  }
}
