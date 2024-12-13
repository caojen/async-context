use std::sync::Arc;
use std::time;
use std::time::Duration;
use tokio::sync::RwLock;
use context_async::{Context, Timer, With};

/// In this lib, our Timer doesn't store any data.
/// However, we usually need to share data with our context.
///
/// In this example, we provide a simple extension implementation.
/// We shore a `User` in our context.

#[derive(Debug)]
#[allow(dead_code)]
struct User {
    username: String,
    user_id: u64,
}

#[derive(Debug, Clone)]
struct MyContext {
    timer: Timer,
    user: Arc<RwLock<Option<User>>>, // use Arc to cheep clone. use RwLock to modify user.
}

#[context_async::async_trait] // re-export from context_async
impl Context for MyContext {
    type SubContext = Self;
    
    fn timer(&self) -> Timer {
        self.timer.clone() // cheep clone
    }

    async fn spawn(&self) -> Self {
        Self {
            timer: self.timer.spawn().await,
            user: self.user.clone(),
        }
    }

    async fn spawn_with_timeout(&self, timeout: Duration) -> Self {
        Self {
            timer: self.timer.spawn_with_timeout(timeout).await,
            user: self.user.clone(),
        }
    }
}

/// when your function needs data, pass your defined Context
async fn update_user(ctx: &MyContext) {
    *ctx.user.write().await = None;
}

/// when your function don't need a data,
/// for example, do some HTTP request, you can use Context as receiver:
async fn fetch_https<Ctx: Context>(ctx: Ctx, url: &str) {
    reqwest::get(url)
        .with(ctx)
        .await
        .unwrap() // -> error from context: timeout or cancelled?
        .unwrap(); // -> error from reqwest
}

#[tokio::main]
async fn main() {
    let ctx = MyContext {
        timer: Timer::with_timeout(time::Duration::from_secs(10)),
        user: Arc::new(RwLock::new(Some(User {
            username: String::from("jack"),
            user_id: 1,
        })))
    };

    update_user(&ctx).await;
    fetch_https(ctx.clone(), "https://www.baidu.com").await;
}
