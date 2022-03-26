use lowestbins::fetch::fetch_auctions;
use lowestbins::server::start_server;
use lowestbins::util::set_interval;
use simplelog::*;
use std::fs::File;
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("creating logger");
    CombinedLogger::init(vec![
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("info.log")?,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create("debug.log")?,
        ),
    ])?;

    set_interval(
        || async {
            if let Err(e) = fetch_auctions().await {
                println!("Error occured while fetching auctions {e:?}")
            }
        },
        Duration::from_secs(3000),
    );

    start_server().await?;
    Ok(())
}
