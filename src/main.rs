use std::{env, fs, process};

use futures::Future;
use lowestbins::{
    fetch::{fetch_auctions, get},
    server::start_server,
    AUCTIONS, CONFIG,
};
use tokio::{time, time::Duration};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

static LOGO: &str = include_str!(concat!(env!("OUT_DIR"), "/logo.txt"));
static SOURCE: &str = "https://github.com/Tricked-dev/lowestbins";

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let now = time::Instant::now();
    get(1).await?;
    println!(
        "{LOGO}\nLoaded {} auctions from save\nMade by Tricked-dev - source: {SOURCE}\nSpeed: {:?}\nOverwrites {:?}, Save To Disk: {}, Update Seconds: {}",
        AUCTIONS.lock().unwrap().len(),
        now.elapsed(),
        &CONFIG.overwrites,
        &CONFIG.save_to_disk,
        &CONFIG.update_seconds,
    );
    let subscriber = FmtSubscriber::builder().with_max_level(Level::DEBUG).finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    set_interval(
        || async {
            if let Err(e) = fetch_auctions().await {
                println!("Error occured while fetching auctions {e:?}")
            }
        },
        Duration::from_secs(CONFIG.update_seconds),
    );

    if CONFIG.save_to_disk {
        ctrlc::set_handler(move || {
            fs::write(
                "auctions.json",
                serde_json::to_string_pretty(&*AUCTIONS.lock().unwrap()).unwrap(),
            )
            .unwrap();
            println!("\nWrote save to disk");
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
