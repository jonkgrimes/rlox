enum ObjectType {
  String,
}

pub trait Object {
  fn object_type(&self) -> ObjectType;
}

pub struct ObjectString {
  data: String,
}
