mod timer;
mod context;
mod error;
mod with;

pub use timer::*;
pub use context::*;
pub use error::*;
pub use with::*;

pub struct TimeChecker(tokio::time::Instant);

impl TimeChecker {
    pub fn new() -> Self {
        Self(tokio::time::Instant::now())
    }

    pub fn not_exceed(&self, duration: tokio::time::Duration) -> bool {
        let diff = tokio::time::Instant::now() - self.0;

        diff < duration
    }
}
