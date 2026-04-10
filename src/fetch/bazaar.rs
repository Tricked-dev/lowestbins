use crate::error::Result;

use serde::Deserialize;

use std::collections::HashMap;

use super::util::get_path;

#[derive(Deserialize, Debug)]
pub struct BazaarResponse {
    #[serde(rename = "products")]
    pub products: HashMap<String, Product>,
}

#[derive(Deserialize, Debug)]
pub struct Product {
    #[serde(rename = "quick_status")]
    pub quick_status: QuickStatus,
}

#[derive(Deserialize, Debug)]
pub struct QuickStatus {
    #[serde(rename = "buyPrice")]
    pub buy_price: f64,
}

pub async fn get_bazaar() -> Result<BazaarResponse> {
    get_path("bazaar").await
}

pub async fn get_bazaar_products() -> Result<HashMap<String, u64>> {
    let bz = get_bazaar().await?;
    let mut map = HashMap::new();
    for (mut key, val) in bz.products.into_iter() {
        if key.starts_with("ENCHANTMENT") {
            let mut split = key.split('_');
            split.next();

            let parts: Vec<&str> = split.collect();

            let (name_parts, level) = parts.split_at(parts.len() - 1);

            key = format!("ENCHANTED_BOOK-{}-{}", name_parts.join("_"), level[0]);
        }
        map.insert(key, val.quick_status.buy_price.round() as u64);
    }
    Ok(map)
}
