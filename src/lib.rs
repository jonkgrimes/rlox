mod chunk;
mod value;
mod vm;

use chunk::{Chunk, OpCode};
use vm::Vm;

pub fn run() {
  let mut chunk = Chunk::new();
  let index = chunk.add_constant(1.2);
  chunk.write_chunk(OpCode::Constant(index), 123);
  chunk.write_chunk(OpCode::Return, 123);
  chunk.disassemble("TEST CHUNK");

  let mut vm = Vm::new(chunk);

  vm.interpret();
}