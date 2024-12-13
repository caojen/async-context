use std::future::Future;
use std::pin::{Pin, pin};
use std::sync::Arc;
use std::task::Poll;
use tokio::{sync, time};
use tokio::sync::RwLock;
use tokio::time::Sleep;
use crate::{Context, Error};

#[derive(Debug, Clone, Default)]
pub struct Timer {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Debug)]
struct Inner {
    expire_at: Option<time::Instant>,
    cancelled: bool,
    cancelled_sender: sync::broadcast::Sender<()>,
    cancelled_receiver: sync::broadcast::Receiver<()>,
    childs: Vec<Timer>,
}

impl Default for Inner {
    fn default() -> Self {
        let (sender, receiver) = sync::broadcast::channel(32);

        Self {
            expire_at: None,
            cancelled: false,
            cancelled_sender: sender,
            cancelled_receiver: receiver,
            childs: Default::default(),
        }
    }
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
        if !inner.cancelled {
            inner.cancelled = true;
            let _ = inner.cancelled_sender.send(());

            for child in &inner.childs {
                child.cancel().await;
            }
        }
    }

    async fn is_cancelled(&self) -> bool {
       self.inner.read().await.cancelled
    }

    async fn is_timeout(&self) -> bool {
        self.inner.read().await.expire_at
            .is_some_and(|expire_at| expire_at < time::Instant::now())
    }

    async fn spawn(&self) -> Self {
        let mut inner = self.inner.write().await;

        let child = Inner {
            expire_at: inner.expire_at,
            ..Default::default()
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
            ..Default::default()
        };

        let child_timer = Self::from(child);
        inner.childs.push(child_timer.clone());

        child_timer
    }

    async fn handle<'a, Fut, Output>(&self, fut: Fut) -> crate::Result<Output>
    where
        Fut: Future<Output = Output> + Send + Sync + 'a
    {
        if self.is_cancelled().await {
            return Err(Error::ContextCancelled);
        }

        let sleep = self.deadline().await
            .map(time::sleep_until)
            .map(Box::pin);

        let mut cancel_receiver = self.cancel_receiver().await;
        let cancel_receiver_fut = cancel_receiver.recv();

        let task = Task {
            cancel_receiver: Box::pin(cancel_receiver_fut),
            sleep,
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
            ..Default::default()
        };

        Self::from(inner)
    }

    async fn cancel_receiver(&self) -> sync::broadcast::Receiver<()> {
        self.inner.read().await.cancelled_receiver.resubscribe()
    }
}

struct Task<'a, Output> {
    fut: Pin<Box<dyn Future<Output = Output> + Send + Sync + 'a>>,
    sleep: Option<Pin<Box<Sleep>>>,
    cancel_receiver: Pin<Box<dyn Future<Output = Result<(), sync::broadcast::error::RecvError>> + Send + Sync + 'a>>,
}

impl<'a, Output> Future for Task<'a, Output> {
    type Output = crate::Result<Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        if let Some(sleep) = this.sleep.as_mut() {
            if let Poll::Ready(_) = pin!(sleep).poll(cx) {
                return Poll::Ready(Err(Error::ContextTimeout));
            }
        }

        if let Poll::Ready(cancel_result) = pin!(&mut this.cancel_receiver).poll(cx) {
            if let Err(e) = cancel_result {
                println!("error when RecvError: {:?}", e);
            }

            return Poll::Ready(Err(Error::ContextCancelled));
        }

        match this.fut.as_mut().poll(cx) {
            Poll::Pending => {
                Poll::Pending
            },
            Poll::Ready(output) => {
                Poll::Ready(Ok(output))
            },
        }
    }
}
