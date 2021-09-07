extern crate serde_json;
extern crate skyblock_rs as skyblock;
extern crate tokio;

use skyblock::*;
use std::env;
use std::error::Error;
use std::vec::Vec;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = env::var("API_KEY")?;
    let mut api = SkyblockApi::singleton(&api_key);

    let mut vec: Vec<Auction> = Vec::new();
    let futa = api.iter_active_auctions(|auction| {
        vec.push(auction);
        Ok(())
    });

    let auctions = futa.await?;

    let json = serde_json::to_string_pretty(&auctions)?;

    println!("{}", json);

    Ok(())
}
