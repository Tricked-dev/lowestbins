use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::str;

use lowestbins::nbt_utils::Item;
use lowestbins::nbt_utils::Pet;
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
    auctions: Vec<Item>,
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
            let r = auctions.get(&auction.name);
            let nbt = &auction.to_nbt().unwrap().i[0];
            let mut id = nbt.tag.extra_attributes.id.clone();
            let count = nbt.count;
            match &nbt.tag.extra_attributes.pet {
                Some(x) => {
                    let v: Pet = serde_json::from_str(x)?;
                    id = format!("PET-{}-{}", v.pet_type, v.tier);
                }
                None => {}
            }
            if id == "ENCHANTED_BOOK" {
                match &nbt.tag.extra_attributes.enchantments {
                    Some(x) => {
                        if x.len() == 1 {
                            for (key, val) in x.iter() {
                                id = format!("ENCHANTED_BOOK-{}-{}", key.to_ascii_uppercase(), val);
                            }
                        }
                    }
                    None => {}
                }
            }
            if id == "POTION" {
                match &nbt.tag.extra_attributes.potion {
                    Some(x) => match &nbt.tag.extra_attributes.potion_level {
                        Some(y) => match &nbt.tag.extra_attributes.enhanced {
                            Some(_) => {
                                id = format!("POTION-{}-{}-ENHANCED", x, y);
                            }
                            None => {
                                id = format!("POTION-{}-{}", x, y);
                            }
                        },
                        None => {
                            id = format!("POTION-{}", x);
                        }
                    },
                    None => {}
                }
            }
            if id == "RUNE" {
                match &nbt.tag.extra_attributes.runes {
                    Some(x) => {
                        if x.len() == 1 {
                            for (key, val) in x.iter() {
                                id = format!("RUNE-{}-{}", key.to_ascii_uppercase(), val);
                            }
                        }
                    }
                    None => {}
                }
            }
            // println!("{} nbt.tag.countT", nbt.tag.Count);
            match r {
                Some(s) => {
                    if s > &auction.starting_bid {
                        auctions.insert(id, auction.starting_bid / count);
                    };
                }
                None => {
                    auctions.insert(id, auction.starting_bid / count);
                }
            }
        }
    }
    for a in 2..r.total_pages {
        // println!("{}", a);
        let page = format!("https://api.hypixel.net/skyblock/auctions?page={}", a);
        let resp = reqwest::get(page).await?.json::<HypixelResponse>().await?;
        let r: HypixelResponse = resp;
        for auction in r.auctions {
            if let Some(true) = auction.bin {
                let r = auctions.get(&auction.name);
                let nbt = &auction.to_nbt().unwrap().i[0];
                let mut id = nbt.tag.extra_attributes.id.clone();
                let count = nbt.count;
                match &nbt.tag.extra_attributes.pet {
                    Some(x) => {
                        let v: Pet = serde_json::from_str(x)?;
                        id = format!("PET-{}-{}", v.pet_type, v.tier);
                    }
                    None => {}
                }
                if id == "ENCHANTED_BOOK" {
                    match &nbt.tag.extra_attributes.enchantments {
                        Some(x) => {
                            if x.len() == 1 {
                                for (key, val) in x.iter() {
                                    id = format!(
                                        "ENCHANTED_BOOK-{}-{}",
                                        key.to_ascii_uppercase(),
                                        val
                                    );
                                }
                            }
                        }
                        None => {}
                    }
                }
                if id == "POTION" {
                    match &nbt.tag.extra_attributes.potion {
                        Some(x) => match &nbt.tag.extra_attributes.potion_level {
                            Some(y) => match &nbt.tag.extra_attributes.enhanced {
                                Some(_) => {
                                    id = format!("POTION-{}-{}-ENHANCED", x, y);
                                }
                                None => {
                                    id = format!("POTION-{}-{}", x, y);
                                }
                            },
                            None => {
                                id = format!("POTION-{}", x);
                            }
                        },
                        None => {}
                    }
                }
                if id == "RUNE" {
                    match &nbt.tag.extra_attributes.runes {
                        Some(x) => {
                            if x.len() == 1 {
                                for (key, val) in x.iter() {
                                    id = format!("RUNE-{}-{}", key.to_ascii_uppercase(), val);
                                }
                            }
                        }
                        None => {}
                    }
                }
                // println!("{} nbt.tag.countT", nbt.tag.Count);
                match r {
                    Some(s) => {
                        if s > &auction.starting_bid {
                            auctions.insert(id, auction.starting_bid / count);
                        };
                    }
                    None => {
                        auctions.insert(id, auction.starting_bid / count);
                    }
                }
            }
        }
    }
    let xs = serde_json::to_string(&auctions).unwrap();
    fs::write("lowestbins.json", xs)?;
    Ok(())
}
