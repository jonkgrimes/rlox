use crate::{Chunk, OpCode};

pub struct Vm {
  chunk: Chunk,
  ip: usize
}

pub enum VmResult {
  Ok,
  CompileError,
  RuntimeError,
}

impl Vm {
  pub fn new(chunk: Chunk) -> Vm {
    Vm { chunk, ip: 0 }
  }

  pub fn interpret(&mut self) -> VmResult {
      self.run()
  }

  fn run(&mut self) -> VmResult {
    for op in self.chunk.code {
      match op {
        OpCode::Return => return VmResult::Ok,
        _ => return VmResult::RuntimeError
      }
    }
  }
}