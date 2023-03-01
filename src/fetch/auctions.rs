use crate::{
    error::Result,
    nbt_utils::{Item, Pet},
    webhook::*,
};

use dashmap::DashMap;
use serde::Deserialize;

use super::util::get_path;

#[derive(Deserialize)]
pub struct HypixelResponse {
    #[serde(rename = "totalPages")]
    pub total_pages: i64,
    #[serde(rename = "auctions")]
    pub auctions: Vec<Item>,
}

pub async fn get_auctions_page(page: i64) -> Result<HypixelResponse> {
    get_path(&format!("auctions?page={page}")).await
}

pub fn parse_auctions(auctions: Vec<Item>, map: &DashMap<String, u64>) -> Result<()> {
    for auction in auctions.iter() {
        if auction.bin {
            let nbt = &auction.to_nbt()?.i[0];
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
                "POTION" => match &nbt.tag.extra_attributes.potion {
                    Some(x) => match &nbt.tag.extra_attributes.potion_level {
                        Some(y) => {
                            if nbt.tag.extra_attributes.enhanced {
                                id = format!("POTION-{}-{}-ENHANCED", x.to_ascii_uppercase(), y);
                            } else {
                                id = format!("POTION-{}-{}", x.to_ascii_uppercase(), y);
                            }
                        }
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
                                id = format!("ATTRIBUTE_SHARD-{}-{}", key.to_ascii_uppercase(), val);
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
    Ok(())
}

pub async fn get_auctions(page: i64, auctions: &DashMap<String, u64>) -> Result<()> {
    let res = get_auctions_page(page).await;
    match res {
        Ok(res) => {
            let map = DashMap::new();
            parse_auctions(res.auctions, &map)?;

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
            send_webhook_text(&format!("Error: {e:?}")).await?;
        }
    };
    Ok(())
}
