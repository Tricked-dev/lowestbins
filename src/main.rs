use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::str;

use lowestbins::nbt_utils::Item;
use lowestbins::nbt_utils::Pet;
use lowestbins::server::start_server;

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
            match id.as_str() {
                "ENCHANTED_BOOK" => {
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
                },
                "POTION" => {
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
                },
                "RUNE" => {
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

                _ => unimplemented!()
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
    println!("writing file");
    fs::write("lowestbins.json", xs)?;
    start_server().await;

    Ok(())
}
