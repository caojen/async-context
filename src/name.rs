use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy)]
pub struct Name(u64);

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

impl Name {
    #[inline]
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}
