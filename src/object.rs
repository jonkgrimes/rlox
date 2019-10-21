use std::fmt;

pub enum Object {
  String(ObjectString),
}

impl fmt::Display for Object {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Object::String(value) => write!(f, "{}", value),
    }
  }
}

pub struct ObjectString {
  data: String,
}

impl ObjectString {
  pub fn new(source: &str) -> Self {
    let string = String::from(source);
    println!("string = {:p}", &string);
    ObjectString {
      data: String::from(source),
    }
  }
}

impl ObjectString {
  pub fn concat(a: &ObjectString, b: &ObjectString) -> ObjectString {
    println!("a.data {}", &a.data);
    println!("a.data {:p}", &a.data);
    println!("b.data {}", &b.data);
    println!("b.data {:p}", &b.data);
    let mut new_string = String::from(&b.data);
    println!("{}", new_string);
    new_string.push_str(&a.data);
    println!("{}", new_string);
    ObjectString::new(&new_string)
  }
}

impl fmt::Display for ObjectString {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "\"{}\"", self.data)
  }
}
