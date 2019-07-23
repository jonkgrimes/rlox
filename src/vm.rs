use crate::compiler::compile;
use crate::value::Value;
use crate::{Chunk, OpCode};

const STACK_MAX: usize = 256;

macro_rules! bin_op {
  ( $self:ident, $op:tt ) => {{
    let a = $self.pop();
    let b = $self.pop();
    $self.push(a $op b);
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
  RuntimeError,
}

impl Vm {
  pub fn new() -> Vm {
    Vm {
      chunk: Chunk::new(),
      ip: 0,
      stack: [0f32; STACK_MAX],
      stack_top: 0,
    }
  }

  pub fn interpret(&mut self, source: &str) -> VmResult {
    compile(source);
    self.run()
  }

  fn run(&mut self) -> VmResult {
    loop {
      let op_code = &self.chunk.code[self.ip];

      if cfg!(feature = "debug") {
        println!("        ");
        for i in 0..self.stack_top {
          println!("[{}]", self.stack[i]);
        }
        println!("        ");
        op_code.disassemble_instruction(&self.chunk, self.ip);
      }

      match op_code {
        OpCode::Return => {
          println!("{}", self.pop());
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
          let value = -self.pop();
          self.push(value);
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
}
