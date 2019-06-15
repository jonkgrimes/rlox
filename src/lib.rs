mod chunk;
mod value;

use chunk::{Chunk, OpCode};

pub fn run() {
  let mut chunk = Chunk::new();
  let index = chunk.add_constant(1.2);
  chunk.write_chunk(OpCode::Constant(index), 123);
  chunk.write_chunk(OpCode::Return, 123);
  chunk.disassemble("TEST CHUNK");
}