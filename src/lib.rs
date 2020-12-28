use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;

use colored::*;
use rustyline::error::ReadlineError;
use rustyline::Editor;

mod chunk;
mod closure;
mod compiler;
mod function;
mod native_function;
mod op_code;
mod scanner;
mod token;
mod upvalue;
mod value;
mod vm;

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
        VmResult::SyntaxError => std::process::exit(65),
        VmResult::RuntimeError(error_message) => {
            let message = format!("Lox::RuntimeError: {}", error_message);
            eprintln!("{}", message.red());
            std::process::exit(70)
        }
        VmResult::Ok => std::process::exit(0),
    }
}

fn interpret(source: &str) -> VmResult {
    let mut vm = Vm::new();
    vm.interpret(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_file(path: &str) -> String {
        let file = File::open(path).ok().expect("Couldn't find test file");
        let mut buf_reader = BufReader::new(file);
        let mut source = String::new();
        buf_reader
            .read_to_string(&mut source)
            .ok()
            .expect("Couldn't read test file");
        source
    }

    #[test]
    fn comments() {
        let source = test_file("test/test-1.lox");
        let result = interpret(&source);
        assert_eq!(result, VmResult::Ok);
    }

    #[test]
    fn printing() {
        let source = test_file("test/test-2.lox");
        let result = interpret(&source);
        assert_eq!(result, VmResult::Ok);
    }

    #[test]
    fn global_variable_assignment() {
        let source = test_file("test/test-3.lox");
        let result = interpret(&source);
        assert_eq!(result, VmResult::Ok);
    }

    #[test]
    fn string_addition() {
        let source = test_file("test/test-4.lox");
        let result = interpret(&source);
        assert_eq!(result, VmResult::Ok);
    }

    #[test]
    fn global_variable_reassignment() {
        let source = test_file("test/test-5.lox");
        let result = interpret(&source);
        assert_eq!(result, VmResult::Ok);
    }

    #[test]
    fn integer_addition() {
        let source = test_file("test/test-6.lox");
        let result = interpret(&source);
        assert_eq!(result, VmResult::Ok);
    }

    #[test]
    fn unlike_types_additon_error() {
        let source = test_file("test/test-6-error.lox");
        let result = interpret(&source);
        assert_eq!(
            result,
            VmResult::RuntimeError("Operand must be a number.".to_string())
        );
    }

    #[test]
    fn more_string_concatenation() {
        let source = test_file("test/test-7.lox");
        let result = interpret(&source);
        assert_eq!(result, VmResult::Ok);
    }

    #[test]
    fn operation_ordering_error() {
        let source = test_file("test/test-8.lox");
        let result = interpret(&source);
        assert_eq!(result, VmResult::Ok);
    }

    #[test]
    fn even_more_string_concatenation() {
        let source = test_file("test/test-9.lox");
        let result = interpret(&source);
        assert_eq!(result, VmResult::Ok);
    }

    #[test]
    fn local_scoping() {
        let source = test_file("test/test-10.lox");
        let result = interpret(&source);
        assert_eq!(result, VmResult::Ok);
    }

    #[test]
    fn if_statements() {
        let source = test_file("test/test-11.lox");
        let result = interpret(&source);
        assert_eq!(result, VmResult::Ok);
    }

    #[test]
    fn variables_in_conditionals() {
        let source = test_file("test/test-12.lox");
        let result = interpret(&source);
        assert_eq!(result, VmResult::Ok);
    }

    #[test]
    fn else_statements() {
        let source = test_file("test/test-13.lox");
        let result = interpret(&source);
        assert_eq!(result, VmResult::Ok);
    }

    #[test]
    fn boolean_logic() {
        let source = test_file("test/test-14.lox");
        let result = interpret(&source);
        assert_eq!(result, VmResult::Ok);
    }

    #[test]
    fn while_loop() {
        let source = test_file("test/test-15.lox");
        let result = interpret(&source);
        assert_eq!(result, VmResult::Ok);
    }

    #[test]
    fn for_loop() {
        let source = test_file("test/test-16.lox");
        let result = interpret(&source);
        assert_eq!(result, VmResult::Ok);
    }

    #[test]
    fn functions() {
        let source = test_file("test/test-17.lox");
        let result = interpret(&source);
        assert_eq!(result, VmResult::Ok);
    }

    #[test]
    fn calling_functions() {
        let source = test_file("test/test-18.lox");
        let result = interpret(&source);
        assert_eq!(result, VmResult::Ok);
    }

    #[test]
    fn passing_arguments() {
        let source = test_file("test/test-19.lox");
        let result = interpret(&source);
        assert_eq!(result, VmResult::Ok);
    }

    #[test]
    fn multiple_calls() {
        let source = test_file("test/test-20.lox");
        let result = interpret(&source);
        assert_eq!(result, VmResult::Ok);
    }

    #[test]
    fn returning_values() {
        let source = test_file("test/test-21.lox");
        let result = interpret(&source);
        assert_eq!(result, VmResult::Ok);
    }

    #[test]
    fn stack_trace() {
        let source = test_file("test/test-22.lox");
        let result = interpret(&source);
        assert_eq!(
            result,
            VmResult::RuntimeError("An error occurred calling a function.".to_string())
        );
    }
}
