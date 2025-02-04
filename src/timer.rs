use std::future::Future;
use std::pin::{Pin, pin};
use std::sync::Arc;
use std::task::Poll;
use std::time;
use log::error;
use tokio::sync;
use tokio::sync::RwLock;
use tokio::time::Sleep;
use crate::{Context, Error};
#[cfg(feature = "name")]
use crate::name::Name;

/// The [`Timer`] structure is the default [`Context`].
#[derive(Debug, Clone)]
pub struct Timer {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Debug)]
struct Inner {
    #[cfg(feature = "name")]
    name: Name,
    expire_at: Option<time::Instant>,
    cancelled: bool,
    cancelled_sender: sync::broadcast::Sender<()>,
    cancelled_receiver: sync::broadcast::Receiver<()>,
    childs: Vec<Timer>,
}

impl Inner {
    fn new() -> Self {
        let (sender, receiver) = sync::broadcast::channel(32);

        #[cfg(feature = "name")]
        let name = Name::default();

        #[cfg(feature = "tracing")]
        {
            #[cfg(feature = "name")]
            tracing::trace!(context_new=name.as_u64());

            #[cfg(not(feature = "name"))]
            tracing::trace!(context_new="");
        }

        Self {
            #[cfg(feature = "name")]
            name,
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
    type SubContext = Self;
    
    fn timer(&self) -> Timer {
        (*self).clone()
    }

    #[cfg(feature = "name")]
    async fn name(&self) -> Name {
        self.inner.read().await.name
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

        let mut child = Inner::new();
        child.expire_at = inner.expire_at;

        #[cfg(feature = "tracing")]
        {
            #[cfg(feature = "name")]
            tracing::trace!(context_spawn=inner.name.as_u64(), child=child.name.as_u64(), expire_at=?child.expire_at);
            #[cfg(not(feature = "name"))]
            tracing::trace!(context_spawn="", expire_at=?child.expire_at)
        }

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

        let mut child = Inner::new();
        child.expire_at = child_expire_at;

        #[cfg(feature = "tracing")]
        {
            #[cfg(feature = "name")]
            tracing::trace!(context_spawn=inner.name.as_u64(), with_timeout=?timeout, child=child.name.as_u64(), expire_at=?child.expire_at);
            #[cfg(not(feature = "name"))]
            tracing::trace!(context_spawn="", with_timeout=?timeout, expire_at=?child.expire_at)
        }

        let child_timer = Self::from(child);
        inner.childs.push(child_timer.clone());

        child_timer
    }

    async fn handle<'a, Fut, Output>(&self, fut: Fut) -> crate::Result<Output>
    where
        Fut: Future<Output = Output> + Send + 'a
    {
        if self.is_cancelled().await {
            return Err(Error::ContextCancelled);
        } else if self.is_timeout().await {
            return Err(Error::ContextTimeout);
        }

        let sleep = self.deadline().await
            .map(tokio::time::Instant::from_std)
            .map(tokio::time::sleep_until)
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
    /// Create a default, independent timer with no time duration limit.
    #[inline]
    pub fn background() -> Self {
        Self::from(Inner::new())
    }

    /// Create a default, independent timer with no time duration limit.
    #[inline]
    pub fn todo() -> Self {
        Self::background()
    }

    /// Specify the maximum execution duration for the `Timer`.
    #[inline]
    pub fn with_timeout(timeout: time::Duration) -> Self {
        let mut inner = Inner::new();
        inner.expire_at = Some(time::Instant::now() + timeout);

        Self::from(inner)
    }

    /// Specify the maximum execution duration for the `Timer`, in seconds.
    #[inline]
    pub fn in_seconds(secs: u64) -> Self {
        Self::with_timeout(time::Duration::from_secs(secs))
    }

    /// Specify the maximum execution duration for the `Timer`, in seconds.
    pub fn in_milliseconds(millis: u64) -> Self {
        Self::with_timeout(time::Duration::from_millis(millis))
    }

    async fn cancel_receiver(&self) -> sync::broadcast::Receiver<()> {
        self.inner.read().await.cancelled_receiver.resubscribe()
    }
}

struct Task<Output, Fut, CErr, CFut>
where
    Fut: Future<Output = Output> + Send,
    CErr: ::std::error::Error,
    CFut: Future<Output = Result<(), CErr>> + Send,
{
    fut: Pin<Box<Fut>>,
    sleep: Option<Pin<Box<Sleep>>>,
    cancel_receiver: Pin<Box<CFut>>,
}

impl<Output, Fut, CErr, CFut> Future for Task<Output, Fut, CErr, CFut>
where
    Fut: Future<Output = Output> + Send,
    CErr: std::error::Error,
    CFut: Future<Output = Result<(), CErr>> + Send,
{
    type Output = crate::Result<Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        if let Some(sleep) = this.sleep.as_mut() {
            if pin!(sleep).poll(cx).is_ready() {
                return Poll::Ready(Err(Error::ContextTimeout));
            }
        }

        if let Poll::Ready(cancel_result) = pin!(&mut this.cancel_receiver).poll(cx) {
            if let Err(e) = cancel_result {
                error!("BUG: error when RecvError: {:?}", e);
            }

            return Poll::Ready(Err(Error::ContextCancelled));
        }

        this.fut.as_mut().poll(cx)
            .map(|r| Ok(r))
    }
}

#[cfg(feature = "actix-web-from-request")]
impl actix_web::FromRequest for Timer {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(_: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        Box::pin(async {
            Ok(Timer::background())
        })
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        #[cfg(feature = "tracing")]
        {
            #[cfg(feature = "name")]
            tracing::trace!(context_drop=self.name.as_u64(), cancelled=self.cancelled, timeout=?self.expire_at);

            #[cfg(not(feature = "name"))]
            tracing::trace!(context_drop="", cancelled=self.cancelled, timeout=?self.expire_at);
        }
    }
}
