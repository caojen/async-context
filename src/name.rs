use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy)]
pub struct Name(pub u64);

impl Default for Name {
    fn default() -> Self {
        Self(rand::random())
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_string())
    }    
}
