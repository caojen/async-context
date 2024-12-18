use std::future::Future;
use std::time;
#[cfg(feature = "name")]
use crate::name::Name;
use crate::Timer;

/// The [`Context`] trait defines the required methods for `Context`.
/// It can define a duration, be cancellable, and immediately cancel
/// async functions if the processing time exceeds the allowed
/// duration without being cancelled. Additionally, the `Context`
/// can spawn child contexts.
///
/// # Examples
/// ```
/// use context_async::{Context, Timer};
/// async fn my_function() {}
///
/// # tokio_test::block_on(async {
/// let timer = Timer::background(); // a new context without duration limit///
/// timer.handle(my_function()).await.unwrap();
/// # });
/// ```
///
/// In addition to using `handle`, you can also handle it
/// with the `future.with()` method by simply importing the `With` trait.
///
/// ```
/// use context_async::{With, Context, Timer};
/// async fn my_function() {}
///
/// let timer = Timer::todo(); // same as background().
/// # tokio_test::block_on(async {
/// my_function()
///     .with(timer)
///     .await
///     .unwrap();
/// # });
/// ```
#[async_trait::async_trait]
pub trait Context: Clone + Send + Sync {
    type SubContext: Context;

    /// return the basic [`Timer`].
    fn timer(&self) -> Timer;

    /// return the name of this context
    #[cfg(feature = "name")]
    async fn name(&self) -> Name {
        self.timer().name().await
    }

    /// return the deadline [`time::Instant`] of this context.
    /// return [None] when this context doesn't have deadline.
    async fn deadline(&self) -> Option<time::Instant> {
        self.timer().deadline().await
    }

    /// cancel this context, then cancel all its childs.
    async fn cancel(&self) {
        self.timer().cancel().await
    }

    /// check whether this context is cancelled or not.
    async fn is_cancelled(&self) -> bool {
        self.timer().is_cancelled().await
    }

    /// check whether this context is timeout or not.
    async fn is_timeout(&self) -> bool {
        self.timer().is_timeout().await
    }

    /// spawn a new child context.
    ///
    /// When the parent (self) is cancelled (call by [`Self::cancel`]),
    /// the child context will be cancelled too.
    ///
    /// # Example
    /// ```rust
    ///
    /// use context_async::{Context, Timer};
    ///
    /// tokio_test::block_on(async {
    /// let ctx = Timer::background();
    /// let child = ctx.spawn().await;
    /// let child_of_child = child.spawn().await;
    ///
    /// ctx.cancel().await; // <- parent is cancelled.
    ///
    /// assert!(child.is_cancelled().await); // the child is cancelled too.
    /// assert!(child_of_child.is_cancelled().await); // also cancelled.
    /// });
    ///
    /// ```
    async fn spawn(&self) -> Self::SubContext;

    /// spawn a new child context, with a new timeout parameter.
    ///
    /// Same as [`Self::spawn`], when the parent (self) is cancelled (call by [`Self::cancel`]),
    /// the child context will be cancelled too.
    ///
    /// The expire_at instant should not be longer than the parent's expire_at.
    ///
    /// # Note
    /// see [`Self::spawn`] for more examples.
    async fn spawn_with_timeout(&self, timeout: time::Duration) -> Self::SubContext;

    /// spawn a new child context, with a new timeout parameter in seconds.
    async fn spawn_in_seconds(&self, secs: u64) -> Self::SubContext {
        self.spawn_with_timeout(time::Duration::from_secs(secs)).await
    }

    /// spawn a new child context, with a new timeout parameter in milliseconds.
    async fn spawn_in_milliseconds(&self, millis: u64) -> Self::SubContext {
        self.spawn_with_timeout(time::Duration::from_millis(millis)).await
    }

    /// handle a future
    ///
    /// # Examples
    /// ```rust
    /// use context_async::{Context, Timer};
    ///
    /// # tokio_test::block_on(async {
    ///  let ctx = Timer::background();
    ///  let task = tokio::time::sleep(tokio::time::Duration::from_secs(1));
    ///
    ///  ctx.handle(task).await.unwrap();
    /// # });
    ///
    /// ```
    async fn handle<'a, Fut, Output>(&self, fut: Fut) -> crate::Result<Output>
    where
        Fut: Future<Output = Output> + Send + 'a
    {
        self.timer().handle(fut).await
    }
}

#[async_trait::async_trait]
impl<T: Context> Context for &T {
    type SubContext = T::SubContext;

    fn timer(&self) -> Timer {
        (*self).timer()
    }

    async fn spawn(&self) -> T::SubContext {
        (*self).spawn().await
    }

    async fn spawn_with_timeout(&self, timeout: time::Duration) -> T::SubContext {
        (*self).spawn_with_timeout(timeout).await
    }
}
