extern crate rustyline;

use std::error::Error;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;

use rustyline::error::ReadlineError;
use rustyline::Editor;

mod chunk;
mod compiler;
mod function;
mod object;
mod op_code;
mod scanner;
mod token;
mod value;
mod vm;

use chunk::Chunk;
use op_code::OpCode;
use vm::{Vm, VmResult};

pub fn repl() -> io::Result<()> {
  let mut rl = Editor::<()>::new();
  rl.load_history("~/.lox_history").ok();
  loop {
    let readline = rl.readline("lox > ");
    match readline {
      Ok(line) => {
        rl.add_history_entry(&line);
        interpret(&line);
      }
      Err(ReadlineError::Interrupted) => {
        println!("Exiting...");
        break;
      }
      Err(err) => {
        eprintln!("Unrecoverable error: {:?}", err);
        break;
      }
    }
  }

  rl.save_history("~/.lox_history").ok();
  Ok(())
}

pub fn run_file(path: &str) -> io::Result<()> {
  let file = File::open(path)?;
  let mut buf_reader = BufReader::new(file);
  let mut contents = String::new();
  buf_reader.read_to_string(&mut contents)?;
  match interpret(&contents) {
    VmResult::CompileError => std::process::exit(65),
    VmResult::RuntimeError(_) => std::process::exit(70),
    VmResult::Ok => std::process::exit(0),
  }
}

fn interpret(source: &str) -> VmResult {
  let mut vm = Vm::new();
  vm.interpret(source)
}
