use crate::object::Object;
use crate::value::Value;

#[derive(Debug)]
pub enum OpCode {
  Return,
  Negate,
  Add,
  Subtract,
  Multiply,
  Divide,
  Not,
  Nil,
  True,
  False,
  Equal,
  Greater,
  Less,
  Print,
  Pop,
  Constant(usize),
  DefineGlobal(usize),
  GetGlobal(usize),
  SetGlobal(usize),
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
      }
      None => {
        "1".to_string() // shouldn't ever happen
      }
    };
    prefix.push_str(&line_number);
    match self {
      OpCode::Return => println!("{} Return", prefix),
      OpCode::Add => println!("{} Add\t", prefix),
      OpCode::Subtract => println!("{} Subtract\t", prefix),
      OpCode::Multiply => println!("{} Multiply\t", prefix),
      OpCode::Divide => println!("{} Divide\t", prefix),
      OpCode::Negate => println!("{} Negate\t", prefix),
      OpCode::Nil => println!("{} Nil", prefix),
      OpCode::True => println!("{} True", prefix),
      OpCode::False => println!("{} False", prefix),
      OpCode::Not => println!("{} Not", prefix),
      OpCode::Equal => println!("{} Equal", prefix),
      OpCode::Greater => println!("{} Greater", prefix),
      OpCode::Less => println!("{} Less", prefix),
      OpCode::Print => println!("{} Print", prefix),
      OpCode::Pop => println!("{} Pop", prefix),
      OpCode::Constant(index) => {
        if let Some(constant) = chunk.constants.get(*index) {
          println!("{} Constant\t{} '{}'", prefix, index, constant);
        }
      }
      OpCode::DefineGlobal(index) => {
        if let Some(constant) = chunk.constants.get(*index) {
          println!("{} DefineGlobal\t{} '{}'", prefix, index, constant);
        }
      }
      OpCode::GetGlobal(index) => {
        if let Some(constant) = chunk.constants.get(*index) {
          println!("{} GetGlobal\t{} '{}'", prefix, index, constant);
        }
      }
      OpCode::SetGlobal(index) => {
        if let Some(constant) = chunk.constants.get(*index) {
          println!("{} SetGlobal\t{} '{}'", prefix, index, constant);
        }
      }
    }
  }
}

pub struct Chunk {
  pub code: Vec<OpCode>,
  lines: Vec<u32>,
  pub constants: Vec<Value>,
  pub objects: Vec<Box<Object>>,
}

impl Chunk {
  pub fn new() -> Chunk {
    Chunk {
      code: Vec::new(),
      constants: Vec::new(),
      objects: Vec::new(),
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
