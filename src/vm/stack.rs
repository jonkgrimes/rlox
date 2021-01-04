use std::ops::RangeBounds;
use std::ops::{Index, IndexMut};
use std::vec::Drain;

use crate::value::Value;

const STACK_MAX: usize = 256;

#[derive(Debug)]
pub struct Stack {
    top: usize,
    stack: Vec<Value>,
}

impl Stack {
    pub fn new() -> Stack {
        Self {
            top: 0,
            stack: Vec::with_capacity(STACK_MAX),
        }
    }

    pub fn reset(&mut self) {
        self.stack.clear();
        self.top = 0;
    }

    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
        self.top += 1;
    }

    pub fn top(&self) -> usize {
        self.top
    }

    pub fn set_top_by_offset(&mut self, offset: usize) {
        self.top -= offset
    }

    pub fn pop(&mut self) -> Value {
        self.top -= 1;
        self.stack.pop().unwrap()
    }

    pub fn peek(&self, distance: usize) -> &Value {
        let peek_index = self.top - distance - 1;
        &self.stack[peek_index]
    }

    pub fn drain<R>(&mut self, range: R) -> Drain<'_, Value>
    where
        R: RangeBounds<usize>,
    {
        let result = self.stack.drain(range);
        result
    }

    pub fn print_stack(&self) {
        println!("======== STACK =======");
        for i in 0..self.top {
            println!("[{}]", self.stack[i]);
        }
    }
}

impl Index<usize> for Stack {
    type Output = Value;

    fn index(&self, index: usize) -> &Self::Output {
        &self.stack[index]
    }
}

impl IndexMut<usize> for Stack {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.stack[index]
    }
}
