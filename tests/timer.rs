use std::time;
use context_async::{Context, Error, TimeChecker, Timer};

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
    assert_eq!(child_2_1_1.error().await, Some(Error::ContextCancelled));
}

#[tokio::test]
async fn test_timer_timeout() {
    let timer = Timer::with_timeout(time::Duration::from_secs(1));
    assert!(!timer.is_timeout().await);

    tokio::time::sleep(time::Duration::from_secs(2)).await;
    assert!(timer.is_timeout().await);

    // check child timeout...
    let timer = Timer::with_timeout(time::Duration::from_secs(10));
    let child = timer.spawn_with_timeout(time::Duration::from_secs(1)).await;

    tokio::time::sleep(time::Duration::from_secs(2)).await;
    assert!(!timer.is_timeout().await);
    assert!(child.is_timeout().await);

    let timer = Timer::with_timeout(time::Duration::from_secs(5));
    let child = timer.spawn_with_timeout(time::Duration::from_secs(10)).await;

    tokio::time::sleep(time::Duration::from_secs(2)).await;
    assert!(!timer.is_timeout().await);
    assert!(!child.is_timeout().await);

    tokio::time::sleep(time::Duration::from_secs(4)).await;
    assert!(timer.is_timeout().await);
    assert!(child.is_timeout().await);
    assert_eq!(timer.error().await, Some(Error::ContextTimeout));
    assert_eq!(child.error().await, Some(Error::ContextTimeout));
}

#[tokio::test]
async fn timer_handle_simple() {
    let timer = Timer::todo();
    timer.handle(tokio::time::sleep(time::Duration::from_secs(1))).await.unwrap();
}

#[tokio::test]
async fn timer_handle_timeout() {
    let tc = TimeChecker::new();
    let timer = Timer::with_timeout(time::Duration::from_secs(1));
    let err = timer.handle(tokio::time::sleep(time::Duration::from_secs(10))).await
        .err().unwrap();

    assert_eq!(err, Error::ContextTimeout);
    assert!(tc.not_exceed(time::Duration::from_millis(1300)));
}

#[tokio::test]
async fn timer_handle_timeout_2() {
    let tc = TimeChecker::new();

    let timer = Timer::with_timeout(time::Duration::from_secs(4));
    let t1 = timer.clone();
    let t2 = timer.clone();

    let t1 = tokio::spawn(async move {
        t1.handle(tokio::time::sleep(time::Duration::from_secs(2))).await.unwrap();
    });

    let t2 = tokio::spawn(async move {
        let err = t2.handle(tokio::time::sleep(time::Duration::from_secs(10))).await.err().unwrap();
        assert_eq!(err, Error::ContextTimeout);
    });

    t1.await.unwrap();
    t2.await.unwrap();

    assert!(tc.not_exceed(time::Duration::from_millis(4300)));
}

#[tokio::test]
async fn timer_partial_cancel() {
    let tc = TimeChecker::new();

    let timer = Timer::with_timeout(time::Duration::from_secs(6));
    let t1 = timer.clone();
    let t2 = timer.clone();

    let t1 = tokio::spawn(async move {
        t1.handle(tokio::time::sleep(time::Duration::from_secs(1))).await.unwrap();
    });

    let t2 = tokio::spawn(async move {
        let err = t2.handle(tokio::time::sleep(time::Duration::from_secs(8))).await.err().unwrap();
        assert_eq!(err, Error::ContextCancelled);
    });

    tokio::time::sleep(time::Duration::from_secs(3)).await;
    timer.cancel().await;

    t1.await.unwrap();
    t2.await.unwrap();

    assert!(tc.not_exceed(time::Duration::from_millis(3300)));
}
