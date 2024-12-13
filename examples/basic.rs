use tokio::time;
use async_context::{Error, Timer, With};

/// In this file, we send a HTTP GET request to https://www.example.com.
/// We use context to handle slow network, and kill the request after 3 seconds.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // give only 3 seconds to finish the request
    let timer = Timer::with_timeout(time::Duration::from_secs(3));

    let url = "https://www.example.com";

    let response = reqwest::Client::new()
        .get(url)
        .send()
        .with(timer.clone()) // add our timer to request future.
        .await;

    match response {
        Ok(Ok(response)) => println!("successfully request: {:?}", response),
        Ok(Err(err)) => println!("request error from reqwest: {:?}", err),
        Err(err) => match err {
            Error::ContextTimeout => println!("request timeout: {}", err),
            Error::ContextCancelled => println!("request cancelled: {}", err),
            _ => unimplemented!(),
        }
    }

    Ok(())
}