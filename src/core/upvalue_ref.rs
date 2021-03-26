use std::fmt;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct UpvalueRef {
    local: bool,
    location: usize
}

impl UpvalueRef {
    pub fn new(local: bool, location: usize) -> Self {
        Self { local, location }
    }

    pub fn local(&self) -> bool {
        self.local
    }
}

impl fmt::Display for UpvalueRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.local {
          write!(f, "Local value")
        } else {
          write!(f, "Upvalue")
        }
    }
}
