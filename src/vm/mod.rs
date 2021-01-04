use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

mod stack;

use crate::{chunk::Chunk};
use crate::closure::Closure;
use crate::compiler::compile;
use crate::function::{Function, FunctionType};
use crate::native_function::NativeFunction;
use crate::value::Value;
use crate::upvalue_ref::UpvalueRef;
use crate::OpCode;
use stack::Stack;

const FRAMES_MAX: usize = 64;

macro_rules! bin_op {
  ( $stack:ident, $op:tt ) => {{
    let a = $stack.peek(0);
    let b = $stack.peek(1);
    if a.is_number() && b.is_number() {
      let a = $stack.pop();
      let b = $stack.pop();
      match b $op a {
        Ok(value) => $stack.push(value),
        Err(msg) => break VmResult::RuntimeError(String::from(msg))
      }
    }
  }};
}

pub struct Vm {
    frames: Vec<CallFrame>,
    frame_count: usize,
}

#[derive(Debug)]
struct CallFrame {
    closure: Closure,
    ip: usize,
    slots: usize,
}

impl CallFrame {
    fn code_at(&self, index: usize) -> &OpCode {
        &self.closure.function.chunk.code[index]
    }

    fn chunk(&self) -> &Chunk {
        &self.closure.function.chunk
    }

    fn get_constant(&self, index: usize) -> Option<&Value> {
        self.closure.function.chunk.constants.get(index)
    }
}

#[derive(Debug, PartialEq)]
pub enum VmResult {
    Ok,
    SyntaxError,
    CompileError,
    RuntimeError(String),
}

impl Vm {
    pub fn new() -> Vm {
        Vm {
            frames: Vec::new(),
            frame_count: 0,
        }
    }

    pub fn interpret(&mut self, source: &str) -> VmResult {
        let function = Function::new("Script", FunctionType::Script);
        let closure = Closure::new(function);
        let mut strings: HashSet<String> = HashSet::new();
        if let Ok(closure) = compile(source, closure, &mut strings) {
            self.frames.push(CallFrame {
                closure,
                ip: 0,
                slots: 0,
            });
            self.run(&mut strings)
        } else {
            return VmResult::CompileError;
        }
    }

    fn frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().unwrap()
    }

    fn frame(&self) -> &CallFrame {
        self.frames.last().unwrap()
    }

    fn run(&mut self, strings: &mut HashSet<String>) -> VmResult {
        let mut globals: HashMap<String, Value> = HashMap::new();
        // Define native functions here
        let clock = clock();
        globals.insert("clock".to_string(), Value::NativeFunction(clock));

        let mut stack: Stack = Stack::new();

        if cfg!(feature = "debug") {
            self.print_iseq();
        }

        loop {
            let ip = self.frame().ip;
            let op_code = &self.frame().code_at(ip).clone();

            if cfg!(feature = "debug") {
                stack.print_stack();
                self.print_globals(&globals);
                OpCode::disassemble_instruction(op_code, &self.frame().chunk(), ip);
            }

            match op_code {
                OpCode::Add => {
                    let a = stack.peek(0).clone();
                    let b = stack.peek(1).clone();
                    match a {
                        Value::String(a) => match b {
                            Value::String(b) => {
                                stack.pop();
                                stack.pop();
                                let mut new_string = String::from(b);
                                new_string.push_str(&a);
                                let new_object =
                                    if let Some(existing_string) = strings.get(&new_string) {
                                        existing_string.to_string()
                                    } else {
                                        new_string
                                    };
                                stack.push(Value::String(new_object));
                            }
                            _ => {
                                break VmResult::RuntimeError(String::from(
                                    "Operand must be a number.",
                                ))
                            }
                        },
                        _ => bin_op!(stack, +),
                    }
                }
                OpCode::Subtract => {
                    bin_op!(stack, -);
                }
                OpCode::Multiply => {
                    bin_op!(stack, *);
                }
                OpCode::Divide => {
                    bin_op!(stack, /);
                }
                OpCode::Negate => {
                    let value = stack.peek(0);
                    if value.is_number() {
                        let a = stack.pop();
                        match -a {
                            Ok(value) => stack.push(value),
                            Err(_) => {
                                break VmResult::RuntimeError(String::from(
                                    "This is unreachable code. How you got here no one knows.",
                                ));
                            }
                        }
                    } else {
                        break VmResult::RuntimeError(String::from("Operand must be a number."));
                    }
                }
                OpCode::Not => {
                    let a = stack.pop();
                    stack.push(Value::Bool(a.is_falsey()));
                }
                OpCode::Nil => {
                    stack.push(Value::Nil);
                }
                OpCode::True => {
                    stack.push(Value::Bool(true));
                }
                OpCode::False => {
                    stack.push(Value::Bool(false));
                }
                OpCode::Equal => {
                    let a = stack.pop();
                    let b = stack.pop();
                    stack.push(Value::Bool(a == b));
                }
                OpCode::Greater => {
                    let a = stack.pop();
                    let b = stack.pop();
                    stack.push(Value::Bool(b > a));
                }
                OpCode::Less => {
                    let a = stack.pop();
                    let b = stack.pop();
                    stack.push(Value::Bool(b < a));
                }
                OpCode::Constant(index) => {
                    let constant = self.frame().get_constant(*index).unwrap();
                    stack.push(constant.clone());
                }
                OpCode::Print => {
                    let value = stack.pop();
                    println!("{}", value);
                }
                OpCode::Pop => {
                    stack.pop();
                }
                OpCode::DefineGlobal(index) => {
                    let constant = self.frame().get_constant(*index);
                    if let Some(constant) = constant {
                        match constant {
                            Value::String(obj) => {
                                let value = stack.peek(0).clone();
                                globals.insert(obj.clone(), value);
                                stack.pop();
                            }
                            _ => {
                                break VmResult::RuntimeError(String::from(
                                    "Cannot resolve variable name.",
                                ))
                            }
                        }
                    }
                }
                OpCode::GetGlobal(index) => {
                    let constant = self.frame().get_constant(*index);
                    if let Some(constant) = constant {
                        match constant {
                            Value::String(s) => {
                                let value = globals.get(s);
                                match value {
                                    Some(value) => stack.push(value.clone()),
                                    _ => {
                                        break VmResult::RuntimeError(
                                            "Cannot resolve variable name.".to_string(),
                                        )
                                    }
                                }
                            }
                            _ => {
                                break VmResult::RuntimeError(
                                    "Cannot resolve variable name.".to_string(),
                                )
                            }
                        }
                    }
                }
                OpCode::SetGlobal(index) => {
                    let constant = self.frame().get_constant(*index);
                    if let Some(constant) = constant {
                        match constant {
                            Value::String(s) => {
                                let value = stack.peek(0).clone();
                                match globals.insert(s.clone(), value) {
                                    Some(_) => (),
                                    None => {
                                        let error = format!("Undefined variable '{}'", s);
                                        globals.remove(s);
                                        break VmResult::RuntimeError(error);
                                    }
                                }
                            }
                            _ => {
                                break VmResult::RuntimeError(
                                    "Cannot resolve variable name.".to_string(),
                                )
                            }
                        }
                    }
                }
                OpCode::SetLocal(index) => {
                    let slots = self.frame().slots;
                    let value = stack.peek(0);
                    stack[slots + *index] = value.clone();
                }
                OpCode::GetLocal(index) => {
                    let slots = self.frame().slots;
                    let value = stack[slots + *index].clone();
                    stack.push(value);
                }
                OpCode::SetUpvalue(index) => {
                    let slots = self.frame().slots;
                    let value = stack.peek(0);
                    stack[slots + *index] = value.clone();
                }
                OpCode::GetUpvalue(index) => {
                    let value = self.frame().closure.upvalues.get(*index).unwrap();
                    stack.push((*value.location_ref()).clone());
                }
                OpCode::JumpIfFalse(offset) => {
                    let value = stack.peek(0);
                    if value.is_falsey() {
                        self.frame_mut().ip += offset;
                    }
                }
                OpCode::Jump(offset) => {
                    self.frame_mut().ip += offset;
                }
                OpCode::Loop(offset) => {
                    self.frame_mut().ip -= offset + 1;
                }
                OpCode::Call(arg_count) => {
                    let value = stack.peek(*arg_count).clone();
                    let (result, function_type) = self.call_value(&mut stack, value, *arg_count);
                    if !result {
                        return VmResult::RuntimeError(
                            "An error occurred calling a function.".to_string(),
                        );
                    } else {
                        if function_type == FunctionType::Function {
                            continue;
                        }
                    }
                }
                OpCode::Closure(index) => {
                    let constant = self.frame().get_constant(*index).unwrap();
                    match constant {
                        Value::Function(function) => {
                            let closure = Closure::new(function.clone());
                            stack.push(Value::Closure(closure));
                        }
                        Value::Closure(closure) => {
                            let mut new_closure = closure.clone();
/*                             for upvalue in &closure.upvalues {
                                if upvalue.local() {
                                    let captured_upvalue = self.capture_upvalue(self.frame().slots + upvalue.index);
                                    new_closure.upvalues.push(captured_upvalue)
                                } else {
                                    let frame_upvalue = self.frame().closure.upvalues.get(upvalue.index).unwrap();
                                    new_closure.upvalues.push(frame_upvalue.clone());
                                }
                            }; */
                            stack.push(Value::Closure(new_closure))
                        },
                        _ => panic!("Received a value that was not a function!"),
                    }
                }
                OpCode::LocalValue(_) => {
                    panic!("Local value opcode was attempted to be executed")
                }
                OpCode::Upvalue(_) => {
                    panic!("Upvalue opcode was attempted to be executed")
                }
                OpCode::Return => {
                    // Get the return value and store temporarily
                    let value = stack.pop();
                    // if we only have one frame, it's the top level script
                    if self.frames.len() == 1 {
                        break VmResult::Ok;
                    }

                    // Need to reset the stack
                    let top = stack.top();
                    // Count number of slots and function
                    let offset = (top - self.frame().slots) + 1;
                    // Drop the references to the slots and function call
                    stack.drain((self.frame().slots - 1)..top);
                    // Set the stack top
                    stack.set_top_by_offset(offset);

                    // Push return of function back onto the stack
                    stack.push(value);

                    // Remove the call frame
                    self.frames.pop();
                }
            }

            self.frame_mut().ip += 1
        }
    }

    fn call_value(
        &mut self,
        stack: &mut Stack,
        callee: Value,
        arg_count: usize,
    ) -> (bool, FunctionType) {
        match callee {
            Value::Closure(closure) => (
                self.call(stack.top(), closure, arg_count),
                FunctionType::Function,
            ),
            Value::NativeFunction(function) => {
                let result = (function.function)();
                stack.pop();
                stack.push(result);
                (true, FunctionType::Native)
            }
            _ => (false, FunctionType::Script),
        }
    }

    fn call(&mut self, stack_top: usize, closure: Closure, arg_count: usize) -> bool {
        let arity = closure.function.arity;
        if arg_count != arity {
            VmResult::RuntimeError(format!(
                "Expected {} arguments but received {}",
                arity, arg_count
            ));
        }

        if self.frames.len() > FRAMES_MAX {
            return false;
        }

        let frame = CallFrame {
            closure,
            ip: 0,
            slots: stack_top - arg_count,
        };

        self.frames.push(frame);

        if cfg!(feature = "debug") {
            self.print_call_frame();
            self.print_iseq();
        }

        true
    }

/*     fn capture_upvalue(&self, index: usize) -> UpvalueRef {
        Upvalue { local: false, index }
    } */

    fn print_call_frame(&mut self) {
        println!("======== FRAME =======");
        println!("{:?}", self.frame());
    }

    fn print_globals(&self, globals: &HashMap<String, Value>) {
        println!("======= GLOBALS =======");
        for (name, value) in globals {
            println!("[{} = {}]", name, value);
        }
    }

    fn print_iseq(&mut self) {
        self.frame_mut().closure.function.disassemble();
    }
}

fn clock() -> NativeFunction {
    let closure = || match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => Value::Number(n.as_secs() as f32),
        _ => Value::Number(0f32),
    };
    NativeFunction {
        name: "clock".to_string(),
        function: closure,
    }
}
