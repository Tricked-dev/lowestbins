use crate::bazaar::get as get_bazaar;
use crate::nbt_utils::{Item, Pet};
use crate::AUCTIONS;
use crate::{HTTP_CLIENT, OVERWRITES};

use anyhow::{anyhow, Result};
use dashmap::DashMap;
use futures::stream::FuturesUnordered;
use futures::FutureExt;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use std::time::Instant;

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
    let mut text = HTTP_CLIENT
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
    #[cfg(feature = "simd")]
    return Ok(simd_json::from_slice(&mut text)?);
    #[cfg(not(feature = "simd"))]
    return Ok(serde_json::from_slice(&text)?);
}

async fn get_auctions(page: i64, auctions: &DashMap<String, u64>) {
    let res = get(page).await;
    match res {
        Ok(res) => {
            let map = DashMap::new();
            parse_hypixel(res.auctions, &map);

            for (x, y) in map.into_iter() {
                if let Some(s) = auctions.get(&x) {
                    if *s < y {
                        continue;
                    };
                }
                auctions.insert(x.to_owned(), y);
            }
        }
        Err(e) => {
            eprintln!("{e:?}");
        }
    };
}

pub async fn get_bazaar_products(auctions: &DashMap<String, u64>) {
    let bz = get_bazaar().await.unwrap();
    let prods = bz.products;
    for (key, val) in prods.iter() {
        auctions.insert(key.to_owned(), val.quick_status.buy_price.round() as u64);
    }
}

pub async fn fetch_auctions() -> Result<()> {
    let start = std::time::Instant::now();
    let hs = get(1).await?;

    let auctions: DashMap<String, u64> = DashMap::new();
    parse_hypixel(hs.auctions, &auctions);

    let futures = FuturesUnordered::new();
    let n = Instant::now();
    for url in 1..hs.total_pages {
        futures.push(get_auctions(url, &auctions).boxed());
    }
    futures.push(get_bazaar_products(&auctions).boxed());

    let _: Vec<_> = futures.collect().await;
    println!("fetch time: {:?}", n.elapsed());
    println!("fetched: {}", auctions.len());

    let mut new_auctions = DashMap::new();
    new_auctions.extend(auctions.clone());
    drop(auctions);
    new_auctions.extend(OVERWRITES.clone());

    let mut auc = AUCTIONS.lock().unwrap();
    auc.extend(new_auctions);
    println!("total: {}", &auc.len());
    println!("Time: {:?}", start.elapsed());
    Ok(())
}
pub fn parse_hypixel(auctions: Vec<Item>, map: &DashMap<String, u64>) {
    for auction in auctions.iter() {
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
                "ATTRIBUTE_SHARD" => match &nbt.tag.extra_attributes.attributes {
                    Some(x) => {
                        if x.len() == 1 {
                            for (key, val) in x.iter() {
                                id =
                                    format!("ATTRIBUTE_SHARD-{}-{}", key.to_ascii_uppercase(), val);
                            }
                        }
                    }
                    None => {}
                },
                _ => {}
            }
            let r = map.get(&id);
            let price = auction.starting_bid / count as u64;
            if let Some(x) = r {
                if *x < price {
                    continue;
                }
            }
            map.insert(id, price);
        }
    }
}
