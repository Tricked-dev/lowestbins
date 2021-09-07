extern crate serde_json;
extern crate skyblock_rs as skyblock;
extern crate tokio;

use skyblock::*;

use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_key = env::var("API_KEY")?;
    let mut api = SkyblockApi::singleton(&api_key);

    let futa = api.get_active_auctions();

    let auctions = futa.await?;

    let json = serde_json::to_string_pretty(&auctions)?;

    println!("{}", json);

    Ok(())
}
