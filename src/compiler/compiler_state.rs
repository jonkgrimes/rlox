use crate::function::{Function, FunctionType};
use super::local::Local;
use super::upvalue::Upvalue;

#[derive(Debug, Clone)]
pub struct CompilerState {
    function: Function,
    function_type: FunctionType,
    enclosing: Option<usize>,
    scope_depth: usize,
    locals: Vec<Local>,
    local_count: usize,
    upvalue_count: usize,
    upvalues: Vec<Upvalue>
}

impl CompilerState {
    pub fn function(&self) -> &Function {
        &self.function
    }

    pub fn function_mut(&mut self) -> &mut Function {
        &mut self.function
    }
}