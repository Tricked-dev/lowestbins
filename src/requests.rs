// Example code that deserializes and serializes the model.
// extern crate serde;
// #[macro_use]
// extern crate serde_derive;
// extern crate serde_json;
//
// use generated_module::[object Object];
//
// fn main() {
//     let json = r#"{"answer": 42}"#;
//     let model: [object Object] = serde_json::from_str(&json).unwrap();
// }

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Welcome {
    pub success: bool,
    pub page: i64,
    pub total_pages: i64,
    pub total_auctions: i64,
    pub last_updated: i64,
    pub auctions: Vec<Auction>,
}

#[derive(Serialize, Deserialize)]
pub struct Auction {
    pub uuid: String,
    pub auctioneer: String,
    pub profile_id: String,
    pub coop: Vec<String>,
    pub start: i64,
    pub end: i64,
    pub item_name: String,
    pub item_lore: String,
    pub extra: String,
    pub category: Category,
    pub tier: Tier,
    pub starting_bid: i64,
    pub item_bytes: String,
    pub claimed: bool,
    pub claimed_bidders: Vec<Option<serde_json::Value>>,
    pub highest_bid_amount: i64,
    pub bin: Option<bool>,
    pub bids: Vec<Bid>,
}

#[derive(Serialize, Deserialize)]
pub struct Bid {
    pub auction_id: String,
    pub bidder: String,
    pub profile_id: String,
    pub amount: i64,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize)]
pub enum Category {
    Accessories,
    Armor,
    Blocks,
    Consumables,
    Misc,
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
