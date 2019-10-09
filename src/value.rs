use std::fmt;

#[derive(Copy, Clone, Debug)]
pub enum Value {
  Nil,
  Bool(bool),
  Number(f32)
}

impl fmt::Display for Value { 
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Value::Nil => write!(f, "nil"),
      Value::Bool(value) => write!(f, "{}", value),
      Value::Number(value) => write!(f, "{}", value),
    }
  }
}