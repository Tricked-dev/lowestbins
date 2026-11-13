use crate::error::Result;

use dashmap::DashMap;
use serde::Deserialize;

use std::collections::HashMap;

use super::util::get_path;

#[derive(Deserialize, Debug)]
pub struct BazaarResponse {
    #[serde(default)]
    pub success: bool,

    #[serde(rename = "lastUpdated", default)]
    pub last_updated: Option<u64>,

    #[serde(rename = "products")]
    pub products: HashMap<String, Product>,
}

#[derive(Deserialize, Debug)]
pub struct Product {
    #[serde(rename = "buy_summary")]
    pub buy_summary: Vec<SummaryEntry>,

    #[serde(default)]
    pub quick_status: Option<QuickStatus>,
}

impl Product {
    #[inline]
    pub fn lowest_buy_price(&self) -> f64 {
        match self.buy_summary.as_slice() {
            [] => 0.0,
            [single] => {
                let p = single.price_per_unit;
                if p.is_finite() { p } else { 0.0 }
            }
            entries => entries
                .iter()
                .map(|s| s.price_per_unit)
                .filter(|p| p.is_finite())
                .reduce(f64::min)
                .unwrap_or(0.0),
        }
    }

    pub fn instant_sell_price(&self) -> Option<f64> {
        self.quick_status
            .as_ref()?
            .buy_price
            .filter(|price| price.is_finite() && *price > 0.0)
    }

    pub fn instant_buy_price(&self) -> Option<f64> {
        self.quick_status
            .as_ref()?
            .sell_price
            .filter(|price| price.is_finite() && *price > 0.0)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QuickStatus {
    pub buy_price: Option<f64>,
    pub sell_price: Option<f64>,
}

#[derive(Deserialize, Debug)]
pub struct SummaryEntry {
    #[serde(rename = "pricePerUnit")]
    pub price_per_unit: f64,
}

pub async fn get_bazaar() -> Result<BazaarResponse> {
    get_path("bazaar").await
}

pub async fn get_bazaar_products(
    bazaar_map: &DashMap<String, f64>,
    instant_buy: &DashMap<String, f64>,
    instant_sell: &DashMap<String, f64>,
) -> Result<Option<u64>> {
    let bz = get_bazaar().await?;

    anyhow::ensure!(bz.success && !bz.products.is_empty(), "Invalid Bazaar response");

    for (mut key, product) in bz.products {
        // preserve naming conv.
        if let Some(rest) = key.strip_prefix("ENCHANTMENT_") {
            if let Some((name, level)) = rest.rsplit_once('_') {
                key = format!("ENCHANTED_BOOK-{name}-{level}");
            }
        }

        if let Some(price) = product.instant_buy_price() {
            instant_buy.insert(key.clone(), price);
        }

        if let Some(price) = product.instant_sell_price() {
            instant_sell.insert(key.clone(), price);
        }

        let price = product.lowest_buy_price();

        if price > 0.0 {
            bazaar_map.insert(key, price);
        }
    }

    anyhow::ensure!(
        !bazaar_map.is_empty() && !instant_sell.is_empty(),
        "Empty Bazaar snapshot"
    );

    Ok(bz.last_updated)
}
