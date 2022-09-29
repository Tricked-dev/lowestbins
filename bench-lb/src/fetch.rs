use std::time::Instant;

use lowestbins::fetch::fetch_auctions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let now = Instant::now();
    for _ in 0..200 {
        fetch_auctions().await?;
    }
    let elapsed = now.elapsed();
    println!("Elapsed: {:?}", elapsed);
    Ok(())
}
