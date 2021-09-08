use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
extern crate base64;

#[derive(Serialize, Deserialize)]
pub struct HypixelResponse {
    #[serde(rename = "success")]
    success: bool,

    #[serde(rename = "page")]
    page: i64,

    #[serde(rename = "totalPages")]
    total_pages: i64,

    #[serde(rename = "totalAuctions")]
    total_auctions: i64,

    #[serde(rename = "lastUpdated")]
    last_updated: i64,

    #[serde(rename = "auctions")]
    auctions: Vec<Auction>,
}

#[derive(Serialize, Deserialize)]
pub struct Auction {
    #[serde(rename = "uuid")]
    uuid: String,

    #[serde(rename = "auctioneer")]
    auctioneer: String,

    #[serde(rename = "profile_id")]
    profile_id: String,

    #[serde(rename = "coop")]
    coop: Vec<String>,

    #[serde(rename = "start")]
    start: i64,

    #[serde(rename = "end")]
    end: i64,

    #[serde(rename = "item_name")]
    item_name: String,

    #[serde(rename = "item_lore")]
    item_lore: String,

    #[serde(rename = "extra")]
    extra: String,

    #[serde(rename = "category")]
    category: Category,

    #[serde(rename = "tier")]
    tier: Tier,

    #[serde(rename = "starting_bid")]
    starting_bid: i64,

    #[serde(rename = "item_bytes")]
    item_bytes: String,

    #[serde(rename = "claimed")]
    claimed: bool,

    #[serde(rename = "claimed_bidders")]
    claimed_bidders: Vec<Option<serde_json::Value>>,

    #[serde(rename = "highest_bid_amount")]
    highest_bid_amount: i64,

    #[serde(rename = "bin")]
    bin: Option<bool>,

    #[serde(rename = "bids")]
    bids: Vec<Bid>,
}

#[derive(Serialize, Deserialize)]
pub struct Bid {
    #[serde(rename = "auction_id")]
    auction_id: String,

    #[serde(rename = "bidder")]
    bidder: String,

    #[serde(rename = "profile_id")]
    profile_id: String,

    #[serde(rename = "amount")]
    amount: i64,

    #[serde(rename = "timestamp")]
    timestamp: i64,
}

#[derive(Serialize, Deserialize)]
pub enum Category {
    #[serde(rename = "accessories")]
    Accessories,

    #[serde(rename = "armor")]
    Armor,

    #[serde(rename = "blocks")]
    Blocks,

    #[serde(rename = "consumables")]
    Consumables,

    #[serde(rename = "misc")]
    Misc,

    #[serde(rename = "weapon")]
    Weapon,
}

#[derive(Serialize, Deserialize)]
pub enum Tier {
    #[serde(rename = "COMMON")]
    Common,

    #[serde(rename = "EPIC")]
    Epic,

    #[serde(rename = "LEGENDARY")]
    Legendary,

    #[serde(rename = "MYTHIC")]
    Mythic,

    #[serde(rename = "RARE")]
    Rare,

    #[serde(rename = "SPECIAL")]
    Special,

    #[serde(rename = "SUPREME")]
    Supreme,

    #[serde(rename = "UNCOMMON")]
    Uncommon,

    #[serde(rename = "VERY_SPECIAL")]
    VerySpecial,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let mut auctions: Vec<Auction> = Vec::new();
    let mut auctions: HashMap<String, i64> = HashMap::new();

    let resp = reqwest::get("https://api.hypixel.net/skyblock/auctions?page=1")
        .await?
        .json::<HypixelResponse>()
        .await?;
    let r: HypixelResponse = resp;

    for auction in r.auctions {
        if let Some(true) = auction.bin {
            let r = auctions.get(&auction.item_name);
            match r {
                Some(s) => {
                    if s > &auction.starting_bid {
                        auctions.insert(auction.item_name, auction.starting_bid);
                    };
                }
                None => {
                    auctions.insert(auction.item_name, auction.starting_bid);
                }
            }
            // if r > &auction.starting_bid {
            //     println!("{} > {}", auction.starting_bid, r);
            //     auctions.insert(auction.item_name, auction.starting_bid);
            // }
        }
    }
    // for auction in auctions {
    //     book_reviews.insert(auction.item_name, auction.starting_bid);
    //     // println!("{}", hematite_nbt::decode(auction.item_bytes))
    //     // let bytes = base64::decode(auction.item_bytes).unwrap();
    //     // println!("{:?}", String::from_utf8_lossy(&bytes));
    //     // println!("{}", auction.item_bytes)
    // }
    // to_writer("bins.json", &book_reviews)?;
    let xs = serde_json::to_string(&auctions).unwrap();
    fs::write("bins.json", xs)?;
    // println!("{:#?}", auctions);
    Ok(())
}
