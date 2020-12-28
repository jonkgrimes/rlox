#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Upvalue {
    pub local: bool,
    pub index: usize,
}
