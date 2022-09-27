use std::env;

use futures::Future;
use lowestbins::fetch::fetch_auctions;
use lowestbins::server::start_server;
use tokio::time;
use tokio::time::Duration;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let seconds = env::var("UPDATE_SECONDS").map_or(60, |f| f.parse().unwrap());
    set_interval(
        || async {
            if let Err(e) = fetch_auctions().await {
                println!("Error occured while fetching auctions {e:?}")
            }
        },
        Duration::from_secs(seconds),
    );

    start_server().await?;
    Ok(())
}
pub fn set_interval<F, Fut>(mut f: F, dur: Duration)
where
    F: Send + 'static + FnMut() -> Fut,
    Fut: Future<Output = ()> + Send + 'static,
{
    let mut interval = time::interval(dur);
    tokio::spawn(async move {
        interval.tick().await;
        loop {
            tokio::spawn(f());
            interval.tick().await;
        }
    });
}
