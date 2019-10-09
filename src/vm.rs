use crate::compiler::compile;
use crate::value::Value;
use crate::{Chunk, OpCode};

const STACK_MAX: usize = 256;

macro_rules! bin_op {
  ( $self:ident, $op:tt ) => {{
    let a = $self.peek(0);
    let b = $self.peek(1);
    match a {
      Value::Number(a_num) => match b {
        Value::Number(b_num) => {
          $self.pop();
          $self.pop();
          $self.push(b $op a);
        },
        _ => {
          break VmResult::RuntimeError(String::from("Operands must be numbers."));
        }
      },
      _ => {
        break VmResult::RuntimeError(String::from("Operands must be numbers."));
      }
    }
  }};
}

pub struct Vm {
  chunk: Chunk,
  ip: usize,
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
          bin_op!(self, +);
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
          match value {
            Value::Number(_) => {
              if let Value::Number(number) = self.pop() {
                self.push(Value::Number(-number));
              } else {
                break VmResult::RuntimeError(String::from(
                  "This is unreachable code. How you got here no one knows.",
                ));
              }
            }
            _ => break VmResult::RuntimeError(String::from("Operand must be a number.")),
          }
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
