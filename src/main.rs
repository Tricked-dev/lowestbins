use lowestbins::fetch::fetch_auctions;
use lowestbins::server::start_server;
use lowestbins::util::set_interval;
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // fetch_auctions().await;

    set_interval(
        || async {
            fetch_auctions().await;
        },
        Duration::from_millis(300000),
    );

    start_server().await;
    Ok(())
}
