use std::collections::HashMap;
use std::fs;

use lowestbins::server::start_server;
use lowestbins::util::{get, parse_hypixel};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let mut auctions: Vec<Auction> = Vec::new();
    let mut auctions: HashMap<String, i64> = HashMap::new();

    let r = get(1).await;
    auctions = parse_hypixel(r.auctions, auctions);
    for a in 2..r.total_pages {
        let r = get(a).await;
        auctions = parse_hypixel(r.auctions, auctions);
    }
    let xs = serde_json::to_string(&auctions).unwrap();
    println!("writing file");
    fs::write("lowestbins.json", xs)?;
    start_server().await;

    Ok(())
}
