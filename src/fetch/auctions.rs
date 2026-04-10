use crate::{
    error::Result,
    nbt_utils::{Item, Pet},
    webhook::*,
};

use std::collections::HashMap;

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

pub fn parse_auctions(auctions: Vec<Item>) -> Result<HashMap<String, u64>> {
    let mut map: HashMap<String, u64> = HashMap::new();
    let mut min_cake_price: Option<u64> = None;
    for auction in auctions.iter() {
        if auction.bin {
            let nbt = &auction.to_nbt()?.i[0];
            let mut id = nbt.tag.extra_attributes.id.clone();
            let count = nbt.count;
            let price = auction.starting_bid / count as u64;
            if let Some(x) = &nbt.tag.extra_attributes.pet {
                let v: Pet = serde_json::from_str(x)?;
                id = format!("PET-{}-{}", v.pet_type, v.tier);
                if let Some(level) = auction.pet_level().filter(|&l| l >= 100) {
                    id = format!("{}-{}", id, (level / 100) * 100);
                }
            }
            match id.as_str() {
                "POTION" => {
                    if let Some(x) = &nbt.tag.extra_attributes.potion {
                        match &nbt.tag.extra_attributes.potion_level {
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
                        }
                    }
                }
                "RUNE" => {
                    if let Some(x) = &nbt.tag.extra_attributes.runes
                        && x.len() == 1 {
                            for (key, val) in x.iter() {
                                id = format!("RUNE-{}-{}", key.to_ascii_uppercase(), val);
                            }
                        }
                }
                "NEW_YEAR_CAKE" => {
                    if let Some(cake_year) = &nbt.tag.extra_attributes.new_years_cake {
                        id = format!("NEW_YEAR_CAKE-{}", cake_year);
                        min_cake_price = Some(min_cake_price.map_or(price, |p| p.min(price)));
                    }
                }

                _ => {}
            }

            if nbt.tag.extra_attributes.base_stat_boost_percentage == Some(50) {
                id.push_str("-PERFECT");
            }

            if let Some(&x) = map.get(&id)
                && x < price
            {
                continue;
            }
            map.insert(id, price);
        }
    }
    if let Some(price) = min_cake_price {
        let existing = map.get("NEW_YEAR_CAKE");
        if existing.is_none_or(|&p| price < p) {
            map.insert("NEW_YEAR_CAKE".to_owned(), price);
        }
    }
    Ok(map)
}

pub async fn get_auctions_page_parsed(page: i64) -> Result<HashMap<String, u64>> {
    let res = get_auctions_page(page).await;
    match res {
        Ok(res) => parse_auctions(res.auctions),
        Err(e) => {
            send_webhook_text(&format!("Error: {e:?}")).await?;
            Ok(HashMap::new())
        }
    }
}
