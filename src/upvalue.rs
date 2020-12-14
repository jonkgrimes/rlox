#[derive(Debug, Clone)]
pub struct Upvalue {
    pub local: bool,
    pub index: usize,
}
