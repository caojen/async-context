use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use async_context::{Context, Error, TimeChecker, Timer, With};

#[tokio::test]
async fn with_simple() {
    let tc = TimeChecker::new();

    let timer = Timer::with_timeout(time::Duration::from_secs(4));

    let err = time::sleep(time::Duration::from_secs(100))
        .with(timer)
        .await
        .err()
        .unwrap();

    assert_eq!(err, Error::ContextTimeout);

    assert!(tc.not_exceed(time::Duration::from_secs(5)));

}

#[derive(Debug, Clone)]
struct DataContext {
    timer: Timer,
    data: Arc<u8>,
}

#[async_trait::async_trait]
impl Context for DataContext {
    fn timer(&self) -> Timer {
        self.timer.clone()
    }

    async fn spawn(&self) -> Self {
        Self {
            timer: self.timer.spawn().await,
            data: self.data.clone(),
        }
    }

    async fn spawn_with_timeout(&self, timeout: Duration) -> Self {
        Self {
            timer: self.timer.spawn_with_timeout(timeout).await,
            data: self.data.clone(),
        }
    }
}

#[tokio::test]
async fn context_with_data() {
    let ctx = DataContext {
        timer: Timer::with_timeout(time::Duration::from_secs(4)),
        data: Arc::new(42),
    };

    let c1 = ctx.clone();
    let t1 = tokio::spawn(async move {
        time::sleep(time::Duration::from_secs(1))
            .with(c1)
            .await
            .unwrap();
    });

    t1.await.unwrap();
}
