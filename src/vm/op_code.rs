use crate::vm::Chunk;
use crate::core::Value;

#[derive(Clone, Copy, Debug)]
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
    GetLocal(usize),
    SetLocal(usize),
    GetUpvalue(usize),
    SetUpvalue(usize),
    JumpIfFalse(usize),
    Jump(usize),
    Loop(usize),
    Call(usize),
    Closure(usize),
    LocalValue(usize),
    Upvalue(usize),
}

impl OpCode {
    pub fn disassemble_instruction(op_code: &OpCode, chunk: &Chunk, offset: usize) {
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
        match op_code {
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
            OpCode::SetLocal(index) => {
                println!("{} SetLocal\t{}", prefix, index);
            }
            OpCode::GetLocal(index) => {
                println!("{} GetLocal\t{}", prefix, index);
            }
            OpCode::GetUpvalue(index) => {
                println!("{} SetLocal\t{}", prefix, index);
            }
            OpCode::SetUpvalue(index) => {
                println!("{} GetLocal\t{}", prefix, index);
            }
            OpCode::JumpIfFalse(jmp) => println!("{} JumpIfFalse offset {}", prefix, jmp),
            OpCode::Jump(jmp) => println!("{} Jump offset {}", prefix, jmp),
            OpCode::Loop(jmp) => println!("{} Loop offset {}", prefix, jmp),
            OpCode::Call(arg_count) => println!("{} Call arg_count {}", prefix, arg_count),
            OpCode::Closure(index) => {
                if let Some(constant) = chunk.constants.get(*index) {
                    if let Value::Closure(closure) = constant {
                        println!("{} Closure", prefix)
/*                         for _ in 0..(closure.function.upvalue_count + 1) {
                            if upvalue.local() {
                                println!("{} Local value\t{} '{}'", prefix, index, constant);
                            } else {
                                println!("{} Upvalue\t{} '{}'", prefix, index, constant);
                            }
                        } */
                    }
                }
            }
            OpCode::LocalValue(index) => {
                println!("{} Local value {}", prefix, index)
            }
            OpCode::Upvalue(index) => {
                println!("{} Upvalue {}", prefix, index)
            }
        }
    }
}
