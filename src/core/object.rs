use crate::core::{Closure, UpvalueRef, NativeFunction};
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Object {
    String(String),
    NativeFunction(NativeFunction),
    Closure(Closure),
    Upvalue(UpvalueRef)
}