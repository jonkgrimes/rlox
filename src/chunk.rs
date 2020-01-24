use crate::op_code::OpCode;
use crate::value::Value;

#[derive(Debug, Clone)]
pub struct Chunk {
  pub code: Vec<OpCode>,
  pub lines: Vec<u32>,
  pub constants: Vec<Value>,
}

impl Chunk {
  pub fn new() -> Chunk {
    Chunk {
      code: Vec::new(),
      constants: Vec::new(),
      lines: Vec::new(),
    }
  }

  pub fn write_chunk(&mut self, op_code: OpCode, line_number: u32) {
    self.lines.push(line_number);
    self.code.push(op_code);
  }

  pub fn add_constant(&mut self, constant: Value) -> usize {
    self.constants.push(constant);
    self.constants.len() - 1
  }

  pub fn disassemble(&self, name: &str) {
    println!("==== {} ====", name);
    for (offset, op_code) in self.code.iter().enumerate() {
      op_code.disassemble_instruction(self, offset);
    }
  }
}
