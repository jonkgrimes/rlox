use std::fmt;

use crate::value::Value;

pub enum OpCode {
  Return,
  Constant(usize)
}

impl OpCode {
  pub fn disassemble_instruction(&self, chunk: &Chunk, offset: usize) {
    let mut prefix = format!("{:04}\t", offset);
    let line_number = match chunk.lines.get(offset) {
      Some(line) => {
        if offset > 0 && line == chunk.lines.get(offset - 1).unwrap() {
          format!("   |")
        } else {
          format!("{:04}", line)
        }
      },
      None => {
        "1".to_string() // shouldn't ever happen
      }
    };
    prefix.push_str(&line_number);
    match self {
      OpCode::Return => println!("{} Return", prefix),
      OpCode::Constant(index) => {
        if let Some(constant) = chunk.constants.get(*index) {
          println!("{} Constant\t{} '{}'", prefix, index, constant);
        }
      }
    }
  }
}

pub struct Chunk {
  code: Vec<OpCode>,
  lines: Vec<u32>,
  constants: Vec<Value>
}

impl Chunk {
  pub fn new() -> Chunk {
    Chunk { code: Vec::new(), constants: Vec::new(), lines: Vec::new() }
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