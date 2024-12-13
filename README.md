# context-async

This lib provide an trait `Context` and a `Timer` to control async function.

It's based on `tokio`.

## Usage

Add this lib as dependencies in `Cargo.toml`.

```toml
[dependencies]
context-async = { version = "0.1" }
```

In your code, you can simple use `Timer`:

```rust
use tokio::time;
use context_async::{Context, Timer, Error, With};

async fn a_heavy_function_or_something_else(a: u8, b: u128) -> Result<(), ()>{
    // ....
    
    Ok(())
}

#[tokio::main]
async fn main() {
    let timer = Timer::with_timeout(time::Duration::from_secs(3));
    
    let fut = a_heavy_function_or_something_else(1, 1024);
    let result = timer.handle(fut).await; // use `handle`
    
    // or:
    // let result = fut.with(timer).await; // trait `With`
    
    let result = match result {
        Err(err) => match err {
            Error::ContextCancelled => "context cancelled",
            Error::ContextTimeout => "context timeout",
        },
        Ok(Err(_)) => "async function error",
        Ok(Ok(_)) => "async function ok",
    };
}
```

`Timer` and `Context` implements `Clone`, which creates a new `Context` (same as itself).

`Timer` and `Context` can spawn children contexts. And ensure the following conditions are met:
1. The life cycle of the child context will not exceed the parent context;
2. If the parent context is cancelled, all child contexts will be cancelled.
3. The cancellation of the child context will be chained to all child contexts, but will not affect the parent context.

for more information, see examples or visit the documentation.
[documentation](https://docs.rs/context-async/latest/context_async/)
