use std::fmt;

use crate::chunk::Chunk;

pub struct Function {
  name: String,
  arity: u32,
  chunk: Chunk,
}

pub enum FunctionType {
  Script,
  Function,
}

impl Function {
  pub fn new(name: &str, arity: u32, chunk: Chunk) -> Function {
    Function {
      name: String::from(name),
      arity,
      chunk,
    }
  }
}

impl fmt::Display for Function {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<fn {}>", self.name)
  }
}
