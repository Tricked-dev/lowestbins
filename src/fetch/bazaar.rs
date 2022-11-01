use crate::error::Result;

use dashmap::DashMap;
use serde::Deserialize;

use std::collections::HashMap;

use super::util::get_path;

#[derive(Deserialize)]
pub struct BazaarResponse {
    #[serde(rename = "products")]
    pub products: HashMap<String, Product>,
}

#[derive(Deserialize)]
pub struct Product {
    #[serde(rename = "quick_status")]
    pub quick_status: QuickStatus,
}

#[derive(Deserialize)]
pub struct QuickStatus {
    #[serde(rename = "buyPrice")]
    pub buy_price: f64,
}
pub async fn get_bazaar() -> Result<BazaarResponse> {
    get_path("bazaar").await
}

pub async fn get_bazaar_products(auctions: &DashMap<String, u64>) -> Result<()> {
    let bz = get_bazaar().await?;
    let prods = bz.products;
    for (mut key, val) in prods.into_iter() {
        if key.starts_with("ENCHANTMENT") {
            let mut split = key.split('_').skip(1);
            let name = split.next().unwrap();
            key = match (split.next(), split.next(), split.next()) {
                (Some(extra), Some(extra2), Some(level)) => {
                    format!("ENCHANTED_BOOK-{name}_{extra}_{extra2}-{level}",)
                }
                (Some(extra), Some(level), None) => {
                    format!("ENCHANTED_BOOK-{name}_{extra}-{level}",)
                }
                (Some(level), None, None) => {
                    format!("ENCHANTED_BOOK-{name}-{level}",)
                }
                _ => format!("ENCHANTED_BOOK-{name}"),
            };
        }
        auctions.insert(key, val.quick_status.buy_price.round() as u64);
    }
    Ok(())
}
