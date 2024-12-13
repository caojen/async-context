use std::future::Future;
use std::pin::{Pin, pin};
use std::sync::Arc;
use std::task::Poll;
use std::time;
use tokio::sync::RwLock;
use crate::Context;

#[derive(Debug, Clone, Default)]
pub struct Timer {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Debug, Default)]
struct Inner {
    expire_at: Option<time::Instant>,
    cancelled: bool,
    childs: Vec<Timer>,
}

#[async_trait::async_trait]
impl Context for Timer {
    fn timer(&self) -> Timer {
        (*self).clone()
    }

    async fn deadline(&self) -> Option<time::Instant> {
        self.inner.read().await
            .expire_at
    }

    async fn cancel(&self) {
        let mut inner = self.inner.write().await;

        inner.cancelled = true;
        for child in &mut inner.childs {
            child.cancel().await;
        }
    }

    async fn is_cancelled(&self) -> bool {
        self.inner.read().await.cancelled
    }

    async fn is_timeout(&self) -> bool {
        let inner = self.inner.read().await;

        inner.expire_at.is_some_and(|expire_at| expire_at < time::Instant::now())
    }

    async fn spawn(&self) -> Self {
        let mut inner = self.inner.write().await;

        let child = Inner {
            expire_at: inner.expire_at,
            cancelled: inner.cancelled,
            childs: Default::default(),
        };

        let child_timer = Self::from(child);
        inner.childs.push(child_timer.clone());

        child_timer
    }

    async fn spawn_with_timeout(&self, timeout: time::Duration) -> Self {
        let mut inner = self.inner.write().await;

        let child_expire_at = time::Instant::now() + timeout;
        let child_expire_at = if let Some(expire_at) = inner.expire_at {
            if child_expire_at > expire_at {
                Some(expire_at)
            } else {
                Some(child_expire_at)
            }
        } else {
            None
        };

        let child = Inner {
            expire_at: child_expire_at,
            cancelled: inner.cancelled,
            childs: Default::default(),
        };

        let child_timer = Self::from(child);
        inner.childs.push(child_timer.clone());

        child_timer
    }

    async fn handle<'a, Fut, Output>(&self, fut: Fut) -> crate::Result<Output>
    where
        Fut: Future<Output = Output> + Send + Sync + 'a
    {
        let task = Task {
            timer: self.clone(),
            fut: Box::pin(fut),
        };

        task.await
    }
}

impl From<Inner> for Timer {
    fn from(value: Inner) -> Self {
        Self { inner: Arc::new(RwLock::new(value)) }
    }
}

impl Timer {
    #[inline]
    pub fn background() -> Self {
        Self::default()
    }

    #[inline]
    pub fn todo() -> Self {
        Self::default()
    }

    #[inline]
    pub fn with_timeout(timeout: time::Duration) -> Self {
        let inner = Inner {
            expire_at: Some(time::Instant::now() + timeout),
            cancelled: false,
            childs: Default::default(),
        };

        Self::from(inner)
    }
}

struct Task<'a, Output> {
    timer: Timer,
    fut: Pin<Box<dyn Future<Output = Output> + Send + Sync + 'a>>
}

impl<'a, Output> Future for Task<'a, Output> {
    type Output = crate::Result<Output>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        // Check `is_cancelled` and `is_timeout`

        match pin!(self.timer.is_timeout()).poll(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(is_timeout) => if is_timeout {
                return Poll::Ready(Err(crate::Error::ContextTimeout));
            }
        }

        match pin!(self.timer.is_cancelled()).poll(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(is_cancelled) => if is_cancelled {
                return Poll::Ready(Err(crate::Error::ContextCancelled));
            }
        }

        match self.fut.as_mut().poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(output) => Poll::Ready(Ok(output)),
        }
    }
}
