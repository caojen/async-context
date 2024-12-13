use std::time;
use async_context::{Context, Timer};

#[tokio::test]
async fn test_timer_deadline() {
    let timer = Timer::background();
    assert!(timer.deadline().await.is_none());

    let timer = Timer::todo();
    assert!(timer.deadline().await.is_none());

    let now = time::Instant::now();
    let timer = Timer::with_timeout(time::Duration::from_secs(10));
    let deadline = timer.deadline().await;
    assert!(deadline.is_some());
    let deadline = deadline.unwrap();

    let diff = deadline - (now + time::Duration::from_secs(10));
    assert!(diff < time::Duration::from_millis(100));
}

#[tokio::test]
async fn test_timer_cancel() {
    let timer = Timer::background();

    let timer_1 = timer.clone();
    let child_1 = timer.spawn().await;
    let child_1_clone = child_1.clone();
    let child_2 = timer_1.spawn().await;
    let child_1_1 = child_1.spawn().await;
    let child_1_2 = child_1_clone.spawn().await;
    let child_1_1_1 = child_1_1.spawn().await;
    let child_2_1_1 = child_2.spawn().await.spawn().await;

    assert!(!timer.is_cancelled().await);
    assert!(!child_2_1_1.is_cancelled().await);

    child_1.cancel().await;
    assert!(!timer.is_cancelled().await);
    assert!(!timer_1.is_cancelled().await);
    assert!(child_1.is_cancelled().await);
    assert!(child_1_clone.is_cancelled().await);
    assert!(child_1_1.is_cancelled().await);
    assert!(child_1_2.is_cancelled().await);
    assert!(child_1_1_1.is_cancelled().await);
    assert!(!child_2.is_cancelled().await);
    assert!(!child_2_1_1.is_cancelled().await);

    child_2_1_1.cancel().await;
    assert!(!timer.is_cancelled().await);
    assert!(!timer_1.is_cancelled().await);
    assert!(child_1.is_cancelled().await);
    assert!(child_1_clone.is_cancelled().await);
    assert!(child_1_1.is_cancelled().await);
    assert!(child_1_2.is_cancelled().await);
    assert!(child_1_1_1.is_cancelled().await);
    assert!(!child_2.is_cancelled().await);
    assert!(child_2_1_1.is_cancelled().await);

    timer_1.cancel().await;
    assert!(timer.is_cancelled().await);
    assert!(timer_1.is_cancelled().await);
    assert!(child_1.is_cancelled().await);
    assert!(child_1_clone.is_cancelled().await);
    assert!(child_1_1.is_cancelled().await);
    assert!(child_1_2.is_cancelled().await);
    assert!(child_1_1_1.is_cancelled().await);
    assert!(child_2.is_cancelled().await);
    assert!(child_2_1_1.is_cancelled().await);
    assert!(child_2_1_1.is_cancelled().await);
}

#[tokio::test]
async fn test_timer_timeout() {
    let timer = Timer::with_timeout(time::Duration::from_secs(1));
    assert!(!timer.is_timeout().await);

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    assert!(timer.is_timeout().await);

    // check child timeout...
    let timer = Timer::with_timeout(time::Duration::from_secs(10));
    let child = timer.spawn_with_timeout(time::Duration::from_secs(1)).await;

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    assert!(!timer.is_timeout().await);
    assert!(child.is_timeout().await);

    let timer = Timer::with_timeout(time::Duration::from_secs(5));
    let child = timer.spawn_with_timeout(time::Duration::from_secs(10)).await;

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    assert!(!timer.is_timeout().await);
    assert!(!child.is_timeout().await);

    tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;
    assert!(timer.is_timeout().await);
    assert!(child.is_timeout().await);
}

