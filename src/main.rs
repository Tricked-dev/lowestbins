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
            File::create("info.log").unwrap(),
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create("debug.log").unwrap(),
        ),
    ])
    .unwrap();
    fetch_auctions().await;

    set_interval(
        || async {
            fetch_auctions().await;
        },
        Duration::from_secs(20),
    );

    start_server().await;
    Ok(())
}
