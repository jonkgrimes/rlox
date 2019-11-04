use std::fmt;

#[derive(Debug, Hash, Eq, PartialEq, PartialOrd)]
pub enum Object {
  String(String),
}

impl fmt::Display for Object {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Object::String(value) => write!(f, "\"{}\"", value),
    }
  }
}
