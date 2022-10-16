use std::{env, fs, process};

use futures_util::Future;
use lowestbins::{
    error::Result,
    fetch::{fetch_auctions, get},
    server::start_server,
    AUCTIONS, CONFIG,
};
use tokio::{time, time::Duration};

static LOGO: &str = include_str!(concat!(env!("OUT_DIR"), "/logo.txt"));
static SOURCE: &str = "https://github.com/Tricked-dev/lowestbins";

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let now = time::Instant::now();
    get(1).await?;
    let res = format!(
        "{LOGO}\nLoaded {} auctions from save\nMade by Tricked-dev - source: {SOURCE}\nSpeed: {:?}\nOverwrites {:?}, Save To Disk: {}, Update Seconds: {}",
        AUCTIONS.lock().unwrap().len(),
        now.elapsed(),
        &CONFIG.overwrites,
        &CONFIG.save_to_disk,
        &CONFIG.update_seconds,
    );
    res.lines().map(|s| tracing::info!("{}", s)).for_each(drop);

    set_interval(
        || async {
            if let Err(e) = fetch_auctions().await {
                tracing::error!("Error occured while fetching auctions {e:?}\n",)
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
            println!();
            tracing::info!("Wrote save to disk\n");
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
