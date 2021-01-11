use crate::token::Token;

#[derive(Debug, Clone)]
pub struct Local {
    name: Token,
    depth: usize,
}
