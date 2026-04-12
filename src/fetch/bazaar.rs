use crate::error::Result;

use dashmap::DashMap;
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
    #[serde(rename = "buy_summary")]
    pub buy_summary: Vec<SummaryEntry>,
}

impl Product {
    #[inline]
    pub fn lowest_buy_price(&self) -> f64 {
        match self.buy_summary.as_slice() {
            [] => 0.0,
            [single] => {
                let p = single.price_per_unit;
                if p.is_finite() {
                    p
                } else {
                    0.0
                }
            }
            entries => entries
                .iter()
                .map(|s| s.price_per_unit)
                .filter(|p| p.is_finite())
                .reduce(f64::min)
                .unwrap_or(0.0),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct SummaryEntry {
    #[serde(rename = "pricePerUnit")]
    pub price_per_unit: f64,
}

pub async fn get_bazaar() -> Result<BazaarResponse> {
    get_path("bazaar").await
}

pub async fn get_bazaar_products(bazaar_map: &DashMap<String, f64>) -> Result<()> {
    let bz = get_path::<BazaarResponse>("bazaar").await?;
    let prods = bz.products;

    for (mut key, val) in prods.into_iter() {
        let price = val.lowest_buy_price();
        // Skip items with no buy orders or invalid prices.
        // This also protects history.rs from tanking averages with 0.0 values,
        // as the item will be absent from the snapshot and won't increment the counter.
        if price <= 0.0 {
            continue;
        }

        if key.starts_with("ENCHANTMENT") {
            let mut split = key.split('_');
            split.next();

            let parts: Vec<&str> = split.collect();

            let (name_parts, level) = parts.split_at(parts.len() - 1);

            key = format!("ENCHANTED_BOOK-{}-{}", name_parts.join("_"), level[0]);
        }

        bazaar_map.insert(key, price);
    }
    Ok(())
}
