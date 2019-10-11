use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum Value {
  Nil,
  Bool(bool),
  Number(f32),
}

impl Value {
  pub fn is_number(&self) -> bool {
    match self {
      Value::Nil => false,
      Value::Bool(_) => false,
      Value::Number(_) => true,
    }
  }

  pub fn is_bool(&self) -> bool {
    match self {
      Value::Nil => false,
      Value::Bool(_) => true,
      Value::Number(_) => false,
    }
  }

  pub fn is_falsey(&self) -> bool {
    match self {
      Value::Nil => true,
      Value::Bool(true) => false,
      Value::Bool(false) => true,
      Value::Number(_) => false,
    }
  }

  fn is_nil(&self) -> bool {
    match self {
      Value::Nil => true,
      Value::Bool(_) => false,
      Value::Number(_) => false,
    }
  }
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

impl Add for Value {
  type Output = Result<Self, &'static str>;

  fn add(self, rhs: Self) -> Result<Self, &'static str> {
    match self {
      Value::Number(value) => match rhs {
        Value::Number(rhs_value) => Ok(Value::Number(value + rhs_value)),
        _ => Err("The right hand side must be a number."),
      },
      _ => Err("Both operands must be a number."),
    }
  }
}

impl Mul for Value {
  type Output = Result<Self, &'static str>;

  fn mul(self, rhs: Self) -> Result<Self, &'static str> {
    match self {
      Value::Number(value) => match rhs {
        Value::Number(rhs_value) => Ok(Value::Number(value * rhs_value)),
        _ => Err("The right hand side must be a number."),
      },
      _ => Err("Both operands must be a number."),
    }
  }
}

impl Div for Value {
  type Output = Result<Self, &'static str>;

  fn div(self, rhs: Self) -> Result<Self, &'static str> {
    match self {
      Value::Number(value) => match rhs {
        Value::Number(rhs_value) => Ok(Value::Number(value / rhs_value)),
        _ => Err("The right hand side must be a number."),
      },
      _ => Err("Both operands must be a number."),
    }
  }
}

impl Sub for Value {
  type Output = Result<Self, &'static str>;

  fn sub(self, rhs: Self) -> Result<Self, &'static str> {
    match self {
      Value::Number(value) => match rhs {
        Value::Number(rhs_value) => Ok(Value::Number(value - rhs_value)),
        _ => Err("The right hand side must be a number."),
      },
      _ => Err("Both operands must be a number."),
    }
  }
}

impl Neg for Value {
  type Output = Result<Self, &'static str>;

  fn neg(self) -> Result<Self, &'static str> {
    match self {
      Value::Number(value) => Ok(Value::Number(-value)),
      _ => Err("You can only negate and number."),
    }
  }
}
