use crate::compiler::compile;
use crate::object::Object;
use crate::value::Value;
use crate::{Chunk, OpCode};

const STACK_MAX: usize = 256;

macro_rules! bin_op {
  ( $self:ident, $op:tt ) => {{
    let a = $self.peek(0);
    let b = $self.peek(1);
    if a.is_number() && b.is_number() {
      let a = $self.pop();
      let b = $self.pop();
      match b $op a {
        Ok(value) => $self.push(value),
        Err(msg) => break VmResult::RuntimeError(String::from(msg))
      }
    }
  }};
}

pub struct Vm {
  chunk: Chunk,
  ip: usize,
  heap: Vec<Box<Object>>,
  stack: [Value; STACK_MAX],
  stack_top: usize,
}

pub enum VmResult {
  Ok,
  CompileError,
  RuntimeError(String),
}

impl Vm {
  pub fn new() -> Vm {
    Vm {
      chunk: Chunk::new(),
      heap: Vec::new(),
      ip: 0,
      stack: [Value::Nil; STACK_MAX],
      stack_top: 0,
    }
  }

  pub fn interpret(&mut self, source: &str) -> VmResult {
    if !compile(source, &mut self.chunk) {
      return VmResult::CompileError;
    }
    self.run()
  }

  fn run(&mut self) -> VmResult {
    loop {
      let op_code = &self.chunk.code[self.ip];

      if cfg!(feature = "debug") {
        self.print_stack();
        op_code.disassemble_instruction(&self.chunk, self.ip);
      }

      match op_code {
        OpCode::Return => {
          println!("=> {}", self.pop());
          break VmResult::Ok;
        }
        OpCode::Add => {
          let a = self.peek(0);
          let b = self.peek(1);
          match a {
            Value::String(a_ptr) => match b {
              Value::String(b_ptr) => {
                self.pop();
                self.pop();
                let boxed_a = unsafe { Box::from_raw(a_ptr) };
                let boxed_b = unsafe { Box::from_raw(b_ptr) };
                match *boxed_a {
                  Object::String(string_a) => match *boxed_b {
                    Object::String(string_b) => {
                      println!("string_a = '{}'", string_a);
                      println!("string_b = '{}'", string_b);
                      let mut new_string = String::from(&string_b);
                      new_string.push_str(&string_a);
                      let new_object = Box::new(Object::String(new_string));
                      self.push(Value::String(Box::into_raw(new_object)));
                    }
                  },
                }
              }
              _ => break VmResult::RuntimeError(String::from("Operand must be a number.")),
            },
            _ => bin_op!(self, +),
          }
        }
        OpCode::Subtract => {
          bin_op!(self, -);
        }
        OpCode::Multiply => {
          bin_op!(self, *);
        }
        OpCode::Divide => {
          bin_op!(self, /);
        }
        OpCode::Negate => {
          let value = self.peek(0);
          if value.is_number() {
            let a = self.pop();
            match -a {
              Ok(value) => self.push(value),
              Err(_) => {
                break VmResult::RuntimeError(String::from(
                  "This is unreachable code. How you got here no one knows.",
                ));
              }
            }
          } else {
            break VmResult::RuntimeError(String::from("Operand must be a number."));
          }
        }
        OpCode::Not => {
          let a = self.pop();
          self.push(Value::Bool(a.is_falsey()));
        }
        OpCode::Nil => {
          self.push(Value::Nil);
        }
        OpCode::True => {
          self.push(Value::Bool(true));
        }
        OpCode::False => {
          self.push(Value::Bool(false));
        }
        OpCode::Equal => {
          let a = self.pop();
          let b = self.pop();
          self.push(Value::Bool(a == b));
        }
        OpCode::Greater => {
          let a = self.pop();
          let b = self.pop();
          self.push(Value::Bool(b > a));
        }
        OpCode::Less => {
          let a = self.pop();
          let b = self.pop();
          self.push(Value::Bool(b < a));
        }
        OpCode::Constant(value) => {
          let constant = self.chunk.constants.get(*value);
          if let Some(constant) = constant {
            self.push(*constant);
          }
        }
      }

      self.ip += 1
    }
  }

  fn reset_stack(&mut self) {
    self.stack_top = 0;
  }

  fn push(&mut self, value: Value) {
    self.stack[self.stack_top] = value;
    self.stack_top += 1;
  }

  fn pop(&mut self) -> Value {
    self.stack_top -= 1;
    let value = self.stack[self.stack_top];
    value
  }

  fn peek(&mut self, distance: usize) -> Value {
    let peek_index = self.stack_top - distance - 1;
    self.stack[peek_index]
  }

  fn print_stack(&self) {
    for i in 0..self.stack_top {
      println!("[{}]", self.stack[i]);
    }
  }
}
