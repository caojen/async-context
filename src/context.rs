use std::future::Future;
use tokio::time;
use crate::Timer;

#[async_trait::async_trait]
pub trait Context: Clone + Send + Sync {
    /// return the basic [`Timer`].
    fn timer(&self) -> Timer;

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
    async fn spawn(&self) -> Self;

    /// spawn a new child context, with a new timeout parameter.
    ///
    /// Same as [`Self::spawn`], when the parent (self) is cancelled (call by [`Self::cancel`]),
    /// the child context will be cancelled too.
    ///
    /// The expire_at instant should not be longer than the parent's expire_at.
    ///
    /// # Note
    /// see [`Self::spawn`] for more examples.
    async fn spawn_with_timeout(&self, timeout: time::Duration) -> Self;

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
        Fut: Future<Output = Output> + Send + Sync + 'a
    {
        self.timer().handle(fut).await
    }
}
