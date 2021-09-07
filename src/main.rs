extern crate serde_json;
extern crate skyblock_rs as skyblock;
extern crate tokio;

use skyblock::*;

use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = env::var("API_KEY")?;
    let mut api = SkyblockApi::singleton(&api_key);

    // api.iter_active_auctions(api);
    // let mut test = "";
    let mut t: std::ops::Fn<Auction>;
    let futa = api.iter_active_auctions(&t);

    let auctions = futa.await?;

    let json = serde_json::to_string_pretty(&auctions)?;

    println!("{}", json);

    Ok(())
}
