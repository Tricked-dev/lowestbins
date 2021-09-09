use serde::{Deserialize, Serialize};
use crate::nbt_utils::{Item, Pet};
use lazy_static::lazy_static;
use std::collections::HashMap;


#[derive(Serialize, Deserialize)]
pub struct HypixelResponse {
    #[serde(rename = "success")]
    success: bool,

    #[serde(rename = "page")]
    pub page: i64,

    #[serde(rename = "totalPages")]
   pub total_pages: i64,

    #[serde(rename = "totalAuctions")]
    pub total_auctions: i64,

    #[serde(rename = "lastUpdated")]
    last_updated: i64,

    #[serde(rename = "auctions")]
   pub  auctions: Vec<Item>,
}

lazy_static! {
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::builder()
        .gzip(true)
        .brotli(true)
        .build()
        .unwrap();
}

pub async fn get(page: i64) -> HypixelResponse {
    let res = HTTP_CLIENT
        .get(format!(
            "https://api.hypixel.net/skyblock/auctions?page={}",
            page
        ))
        .send()
        .await
        .unwrap();
    let text = res.text().await.unwrap();
    return serde_json::from_str(&text).unwrap();
}


pub fn parse_hypixel(auctions:Vec<Item>, mut map: HashMap<String, i64>) -> HashMap<String, i64> {
    for auction in auctions {
        if let Some(true) = auction.bin {
            let nbt = &auction.to_nbt().unwrap().i[0];
            let mut id = nbt.tag.extra_attributes.id.clone();
            let count = nbt.count;

            match &nbt.tag.extra_attributes.pet {
                Some(x) => {
                    let v: Pet = serde_json::from_str(x).unwrap();
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

                _ => {}
            }
            let r = map.get(&id);
        println!("{}", &id);
            // println!("{} nbt.tag.countT", nbt.tag.Count);
            match r {
                Some(s) => {
                    if s > &auction.starting_bid {
                        map.insert(id, auction.starting_bid / count);
                    };
                }
                None => {
                    map.insert(id, auction.starting_bid / count);
                }

            }
        }
    }
    map
}