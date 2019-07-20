use crate::value::Value;
use crate::{Chunk, OpCode};

const STACK_MAX: usize = 256;

pub struct Vm {
  chunk: Chunk,
  ip: usize,
  stack: [Value; STACK_MAX],
  stack_top: usize,
}

pub enum VmResult {
  Noop,
  Ok,
  CompileError,
  RuntimeError,
}

impl Vm {
  pub fn new(chunk: Chunk) -> Vm {
    Vm {
      chunk,
      ip: 0,
      stack: [0f32; STACK_MAX],
      stack_top: 0,
    }
  }

  pub fn interpret(&mut self) -> VmResult {
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
        OpCode::Negate => {
          println!("{}", -self.pop());
        },
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

  fn print_stack(&mut self) {
    println!("        ");
    for i in 0..self.stack_top {
      println!("[{}]", self.stack[i]);
    }
    println!("        ");
  }
}
