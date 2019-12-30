use std::collections::{HashMap, HashSet};

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
  pub chunk: Chunk,
  pub strings: HashSet<String>,
  pub globals: HashMap<Object, Value>,
  ip: usize,
  stack: Vec<Value>,
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
      ip: 0,
      strings: HashSet::new(),
      globals: HashMap::new(),
      stack: Vec::with_capacity(STACK_MAX),
      stack_top: 0,
    }
  }

  pub fn interpret(&mut self, source: &str) -> VmResult {
    if !compile(source, &mut self.chunk, &mut self.strings) {
      return VmResult::CompileError;
    }
    self.run()
  }

  fn run(&mut self) -> VmResult {
    loop {
      let op_code = &self.chunk.code[self.ip];

      if cfg!(feature = "debug") {
        self.print_stack();
        self.print_globals();
        op_code.disassemble_instruction(&self.chunk, self.ip);
      }

      match op_code {
        OpCode::Return => {
          break VmResult::Ok;
        }
        OpCode::Add => {
          let a = self.peek(0).clone();
          let b = self.peek(1).clone();
          match a {
            Value::String(a) => match b {
              Value::String(b) => {
                self.pop();
                self.pop();
                match a {
                  Object::String(string_a) => match b {
                    Object::String(string_b) => {
                      let mut new_string = String::from(string_b);
                      new_string.push_str(&string_a);
                      let new_object = if let Some(existing_string) = self.strings.get(&new_string)
                      {
                        Object::String(existing_string.to_string())
                      } else {
                        Object::String(new_string)
                      };
                      self.push(Value::String(new_object));
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
            self.push(constant.clone());
          }
        }
        OpCode::Print => {
          let value = self.pop();
          println!("{}", value);
        }
        OpCode::Pop => {
          self.pop();
        }
        OpCode::DefineGlobal(index) => {
          let constant = self.chunk.constants.get(*index);
          if let Some(constant) = constant {
            match constant {
              Value::String(obj) => {
                let value = self.peek(0);
                self.globals.insert(obj.clone(), value.clone());
                self.pop();
              }
              _ => break VmResult::RuntimeError(String::from("Cannot resolve variable name.")),
            }
          }
        }
        OpCode::GetGlobal(index) => {
          let constant = self.chunk.constants.get(*index);
          if let Some(constant) = constant {
            match constant {
              Value::String(obj) => {
                let value = self.globals.get(&obj);
                match value {
                  Some(value) => self.push(value.clone()),
                  _ => break VmResult::RuntimeError("Cannot resolve variable name.".to_string()),
                }
              }
              _ => break VmResult::RuntimeError("Cannot resolve variable name.".to_string()),
            }
          }
        }
        OpCode::SetGlobal(index) => {
          let constant = self.chunk.constants.get(*index);
          if let Some(constant) = constant {
            match constant {
              Value::String(obj) => {
                let value = self.peek(0);
                match self.globals.insert(obj.clone(), value.clone()) {
                  Some(_) => (),
                  None => {
                    let error = format!("Undefined variable '{}'", obj);
                    self.globals.remove(&obj);
                    break VmResult::RuntimeError(error);
                  }
                }
              }
              _ => break VmResult::RuntimeError("Cannot resolve variable name.".to_string()),
            }
          }
        }
        OpCode::SetLocal(index) => {
          let i = (*index).clone();
          let value = self.peek(0);
          self.stack[i] = value.clone();
        }
        OpCode::GetLocal(index) => {
          let i = (*index).clone();
          let value = self.stack[i].clone();
          self.push(value);
        }
        OpCode::JumpIfFalse(offset) => {
          let value = self.peek(0);
          if value.is_falsey() {
            self.ip += offset;
          }
        }
        OpCode::Jump(offset) => {
          self.ip += offset;
        }
        OpCode::Loop(offset) => {
          self.ip -= offset + 1;
        }
      }

      self.ip += 1
    }
  }

  fn reset_stack(&mut self) {
    self.stack.clear();
    self.stack_top = 0;
  }

  fn push(&mut self, value: Value) {
    self.stack.push(value);
    self.stack_top += 1;
  }

  fn pop(&mut self) -> Value {
    self.stack_top -= 1;
    self.stack.pop().unwrap()
  }

  fn peek(&self, distance: usize) -> &Value {
    let peek_index = self.stack_top - distance - 1;
    &self.stack[peek_index]
  }

  fn print_stack(&self) {
    println!("======= STACK   =======");
    for i in 0..self.stack_top {
      println!("[{}]", self.stack[i]);
    }
  }

  fn print_globals(&self) {
    println!("======= GLOBALS =======");
    for (name, value) in &self.globals {
      println!("[{} = {}]", name, value);
    }
  }
}
