use std::fmt;

enum ObjectType {
  String,
}

pub trait Object
where
  Self: std::fmt::Display,
{
  fn object_type(&self) -> ObjectType;
}

pub struct ObjectString {
  data: String,
}

impl ObjectString {
  pub fn new(source: &str) -> Self {
    ObjectString {
      data: String::from(source),
    }
  }
}

impl Object for ObjectString {
  fn object_type(&self) -> ObjectType {
    ObjectType::String
  }
}

impl fmt::Display for ObjectString {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.data)
  }
}
