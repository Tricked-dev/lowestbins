use lowestbins::fetch::fetch_auctions;
use lowestbins::server::start_server;
use lowestbins::util::set_interval;
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    set_interval(
        || async {
            if let Err(e) = fetch_auctions().await {
                println!("Error occured while fetching auctions {e:?}")
            }
        },
        Duration::from_secs(60),
    );

    start_server().await?;
    Ok(())
}
