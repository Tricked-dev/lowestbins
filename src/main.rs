use std::{env, fs};

use futures_util::future::join;
use lowestbins::{error::Result, fetch::fetch_auctions, server::start_server, AUCTIONS, CONFIG, SOURCE};
use mimalloc::MiMalloc;
use tokio::{time, time::Duration};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
static LOGO: &str = include_str!(concat!(env!("OUT_DIR"), "/logo.txt"));

pub fn create_basic_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .max_blocking_threads(32)
        .build()
        .unwrap()
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let rt = create_basic_runtime();

    let res = format!(
        "{LOGO}\nLoaded {} auctions from save\nMade by Tricked-dev - source: {SOURCE}\nOverwrites {:?}, Save To Disk: {}, Update Seconds: {}",
        AUCTIONS.lock().len(),
        &CONFIG.overwrites,
        &CONFIG.save_to_disk,
        &CONFIG.update_seconds,
    );
    res.lines().map(|s| tracing::info!("{}", s)).for_each(drop);

    if CONFIG.save_to_disk {
        rt.spawn(async {
            let dur = Duration::from_secs(CONFIG.update_seconds);
            let mut interval = time::interval(dur);
            loop {
                interval.tick().await;
                if !AUCTIONS.is_locked() {
                    match fs::write(
                        "auctions.json",
                        serde_json::to_string_pretty(&*AUCTIONS.lock()).unwrap(),
                    ) {
                        Ok(_) => tracing::debug!("Saved to disk"),
                        Err(_) => tracing::error!(
                            "Failed to save auctions to disk please give write permissions to current directory"
                        ),
                    };
                } else {
                    tracing::error!("Auctions poisoned, not saving to disk");
                }
            }
        });
    }
    rt.spawn(async {
        let dur = Duration::from_secs(CONFIG.update_seconds);
        let mut interval = time::interval(dur);
        interval.tick().await;
        loop {
            // Dont spawn a thread but instead wait for both futures to finish and continue
            join(
                async {
                    if let Err(e) = fetch_auctions().await {
                        tracing::error!("Error occured while fetching auctions {e:?}\n",)
                    }
                },
                interval.tick(),
            )
            .await;
        }
    });

    rt.block_on(start_server())?;
    Ok(())
}
