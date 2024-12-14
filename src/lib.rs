#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![forbid(unsafe_code)]

//! This lib provide an trait [`Context`] and a [`Timer`] to control async function.
//!
//! ## Example
//!
//! Here is a simple example. We use [`Timer`] to control an async function
//! to finish in 5 seconds.
//!
//! ```rust
//! use tokio::time;
//! use context_async::{Context, Timer};
//!
//! async fn a_heavy_async_function(a: u8) -> bool {
//!     return true;
//! }
//!
//! # tokio_test::block_on(async {
//!  let timer = Timer::with_timeout(time::Duration::from_secs(5));
//!
//!  let _ = timer.handle(a_heavy_async_function(0)).await;
//!
//!  // or:
//!
//!  use context_async::With;
//!  let _ = a_heavy_async_function(0).with(&timer).await;
//! # });
//! ```
//!
//! [`Timer`] implements [`Context`], you can pass a [`Context`] to your async function:
//!
//! ```rust
//!
//! use std::time;
//! use context_async::{Context, Error, Timer, With};
//!
//! async fn a_heavy_async_function(a: u8) -> bool {
//!     return true;
//! }
//!
//! async fn my_function<Ctx: Context>(ctx: Ctx) -> Result<bool, Error> {
//!     a_heavy_async_function(0)
//!         .with(ctx)
//!         .await
//! }
//!
//! # tokio_test::block_on(async {
//! let timer = Timer::with_timeout(time::Duration::from_secs(5));
//! let _ = my_function(&timer).await;
//! # })
//!
//! ```
//!
//! ## Error
//!
//! [`Context`] returns [`Error`], one of [`Error::ContextCancelled`] or [`Error::ContextTimeout`].
//!
//! ## Features
//! - `actix-web-from-request`: implement actix-web::FromRequest for [`Timer`].
//! - `name`: create a name for each [`Context`].
//! - `tracing`: enable `tracing` and do `tracing::trace!(...)` logging.

mod timer;
mod context;
mod error;
mod with;
#[cfg(feature = "name")]
mod name;

pub use timer::*;
pub use context::*;
pub use error::*;
pub use with::*;
#[cfg(feature = "name")]
pub use name::*;

pub use async_trait::async_trait;

pub struct TimeChecker(tokio::time::Instant);

impl Default for TimeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeChecker {
    pub fn new() -> Self {
        Self(tokio::time::Instant::now())
    }

    pub fn not_exceed(&self, duration: tokio::time::Duration) -> bool {
        let diff = tokio::time::Instant::now() - self.0;

        diff < duration
    }
}
