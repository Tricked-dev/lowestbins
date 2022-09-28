use std::env;
use std::fs;
use std::process;

use futures::Future;
use lowestbins::fetch::fetch_auctions;
use lowestbins::server::start_server;
use lowestbins::AUCTIONS;
use tokio::time;
use tokio::time::Duration;

static UPDATE_SECONDS: &str = "UPDATE_SECONDS";
static SAVE_TO_DISK: &str = "SAVE_TO_DISK";

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let seconds = env::var(UPDATE_SECONDS).map_or(60, |f| f.parse().unwrap());
    set_interval(
        || async {
            if let Err(e) = fetch_auctions().await {
                println!("Error occured while fetching auctions {e:?}")
            }
        },
        Duration::from_secs(seconds),
    );

    if env::var(SAVE_TO_DISK).unwrap_or_else(|_| "1".to_owned()) != "0" {
        ctrlc::set_handler(move || {
            fs::write(
                "auctions.json",
                serde_json::to_string_pretty(&*AUCTIONS.lock().unwrap()).unwrap(),
            )
            .unwrap();
            println!("Wrote save to disk");
            process::exit(0)
        })?;
    }

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
