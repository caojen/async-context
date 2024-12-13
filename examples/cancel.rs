use async_context::{Context, Timer};

/// In this example, we spawn a few context from a root context, and show how `cancel` works.

async fn a_very_heavy_task() {
    tokio::time::sleep(tokio::time::Duration::from_secs(100)).await;
}

#[tokio::main]
async fn main() {
    // our job should finish in 10 seconds.
    let root = Timer::with_timeout(tokio::time::Duration::from_secs(10));

    let root2 = root.clone(); // clone a context (same as root).
    let root_job = tokio::spawn(async move {
        let result = root2.handle(a_very_heavy_task()).await;
        println!("root2 result: {:?}", result);
    });

    let t1 = root.spawn().await; // spawn a child context.
    let t1_to_cancel = t1.clone(); // t1 will be moved, so the clone it for cancel.
    let t11 = t1.spawn().await; // spawn a child context of t1.
    let t11_to_cancel = t11.clone();

    let j1 = tokio::spawn(async move {
        let result = t1.handle(a_very_heavy_task()).await;
        println!("t1 result: {:?}", result);
    });

    let j11 = tokio::spawn(async move {
        let result = t11.handle(a_very_heavy_task()).await;
        println!("t11 result: {:?}", result);
    });

    // if we cancel t1, then t11 (the child of t1) would be cancelled at the same time.
    t1_to_cancel.cancel().await;
    // it would print that t1 and t11 `ContextCancelled`.
    assert!(t1_to_cancel.is_cancelled().await);
    assert!(t11_to_cancel.is_cancelled().await);

    // but this would NOT affect root, because root is the parent of t1.
    assert!( ! root.is_cancelled().await );

    root_job.await.unwrap();
    j1.await.unwrap();
    j11.await.unwrap();

    // the root job will end with error `ContextTimeout` after 10s.

    // STDOUT:
    // t1 result: Err(ContextCancelled)
    // t11 result: Err(ContextCancelled)
    // root2 result: Err(ContextTimeout)
}
