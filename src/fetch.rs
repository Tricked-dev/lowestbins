use crate::bazaar::get as get_bazaar;
use crate::nbt_utils::{Item, Pet};
use crate::{AUCTIONS, PARRALEL};
use crate::{HTTP_CLIENT, OVERWRITES};

use anyhow::{anyhow, Result};
use futures::{stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize)]
pub struct HypixelResponse {
    #[serde(rename = "page")]
    pub page: i64,
    #[serde(rename = "totalPages")]
    pub total_pages: i64,
    #[serde(rename = "auctions")]
    pub auctions: Vec<Item>,
}

pub async fn get(page: i64) -> Result<HypixelResponse> {
    let text = HTTP_CLIENT
        .get(format!(
            "https://api.hypixel.net/skyblock/auctions?page={}",
            page
        ))
        .send()
        .await
        .map_err(|x| anyhow!(x))?
        .body_bytes()
        .await
        .map_err(|x| anyhow!(x))?;

    Ok(serde_json::from_slice(&text)?)
}

pub async fn fetch_auctions() -> Result<()> {
    let start = std::time::Instant::now();
    let hs = get(1).await?;

    let auctions: Arc<Mutex<HashMap<String, u64>>> =
        Arc::new(Mutex::new(parse_hypixel(hs.auctions, HashMap::new())));

    let bodies = stream::iter(2..hs.total_pages)
        .map(|url| async move {
            let auctions: HashMap<String, u64> = HashMap::new();
            let res = get(url).await;
            match res {
                Ok(res) => Some(parse_hypixel(res.auctions, auctions)),
                Err(e) => {
                    eprintln!("{e:?}");
                    None
                }
            }
        })
        .buffer_unordered(*PARRALEL);

    bodies
        .for_each(|res: Option<HashMap<String, u64>>| async {
            if let Some(res) = res {
                let mut auctions = auctions.lock().unwrap();
                for (x, y) in res.iter() {
                    if let Some(s) = auctions.get(x) {
                        if s < y {
                            continue;
                        };
                    }
                    auctions.insert(x.to_owned(), *y);
                }
            };
        })
        .await;
    let bz = get_bazaar().await?;
    let prods = bz.products;
    let mut auctions = auctions.lock().unwrap();
    for (key, val) in prods.iter() {
        let k = key.to_owned().replace("ENCHANTED_BOOK-", "ENCHANTMENT");
        auctions.insert(k, val.quick_status.buy_price.round() as u64);
    }
    auctions.extend(OVERWRITES.clone());
    let xs = serde_json::to_vec(&*auctions)?;

    let mut auc = AUCTIONS.lock().unwrap();
    println!(
        "size: {}KB\nFetched auctions in {:?}",
        xs.len() / 1000,
        start.elapsed()
    );
    *auc = xs;
    Ok(())
}
pub fn parse_hypixel(auctions: Vec<Item>, mut map: HashMap<String, u64>) -> HashMap<String, u64> {
    for auction in auctions {
        if auction.bin {
            let nbt = &auction.to_nbt().unwrap().i[0];
            let mut id = nbt.tag.extra_attributes.id.clone();
            let count = auction.count;

            match &nbt.tag.extra_attributes.pet {
                Some(x) => {
                    let v: Pet = serde_json::from_str(x).unwrap();
                    id = format!("PET-{}-{}", v.pet_type, v.tier);
                }
                None => {}
            }
            match id.as_str() {
                "POTION" => match &nbt.tag.extra_attributes.potion {
                    Some(x) => match &nbt.tag.extra_attributes.potion_level {
                        Some(y) => match &nbt.tag.extra_attributes.enhanced {
                            true => {
                                id = format!("POTION-{}-{}-ENHANCED", x.to_ascii_uppercase(), y);
                            }
                            false => {
                                id = format!("POTION-{}-{}", x.to_ascii_uppercase(), y);
                            }
                        },
                        None => {
                            id = format!("POTION-{}", x.to_ascii_uppercase());
                        }
                    },
                    None => {}
                },
                "RUNE" => match &nbt.tag.extra_attributes.runes {
                    Some(x) => {
                        if x.len() == 1 {
                            for (key, val) in x.iter() {
                                id = format!("RUNE-{}-{}", key.to_ascii_uppercase(), val);
                            }
                        }
                    }
                    None => {}
                },
                _ => {}
            }
            let r = map.get(&id);
            match r {
                Some(s) => {
                    if s > &auction.starting_bid {
                        map.insert(id, auction.starting_bid / count as u64);
                    };
                }
                None => {
                    map.insert(id, auction.starting_bid / count as u64);
                }
            }
        }
    }
    map
}
