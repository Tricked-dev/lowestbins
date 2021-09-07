use skyblock_rs::*;
use std::env;
use std::vec::Vec;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = env::var("API_KEY")?;
    let mut api = SkyblockApi::singleton(&api_key);

    let mut vec: Vec<Auction> = Vec::new();
    api.iter_active_auctions(|auction| {
        vec.push(auction);
        Ok(())
    }).await.unwrap();

    println!("{:#?}", vec);

    Ok(())
}
