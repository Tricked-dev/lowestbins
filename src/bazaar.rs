use std::collections::HashMap;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json;

use crate::HTTP_CLIENT;

#[derive(Serialize, Deserialize)]
pub struct BazaarResponse {
    #[serde(rename = "success")]
    success: bool,

    #[serde(rename = "lastUpdated")]
    last_updated: i64,

    #[serde(rename = "products")]
    pub products: HashMap<String, Product>,
}

#[derive(Serialize, Deserialize)]
pub struct Product {
    #[serde(rename = "product_id")]
    product_id: String,

    #[serde(rename = "sell_summary")]
    pub sell_summary: Vec<Summary>,

    #[serde(rename = "buy_summary")]
    pub buy_summary: Vec<Summary>,

    #[serde(rename = "quick_status")]
    pub quick_status: QuickStatus,
}

#[derive(Serialize, Deserialize)]
pub struct Summary {
    #[serde(rename = "amount")]
    amount: i64,

    #[serde(rename = "pricePerUnit")]
    pub price_per_unit: f64,

    #[serde(rename = "orders")]
    orders: i64,
}

#[derive(Serialize, Deserialize)]
pub struct QuickStatus {
    #[serde(rename = "productId")]
    product_id: String,

    #[serde(rename = "sellPrice")]
    sell_price: f64,

    #[serde(rename = "sellVolume")]
    sell_volume: i64,

    #[serde(rename = "sellMovingWeek")]
    sell_moving_week: i64,

    #[serde(rename = "sellOrders")]
    sell_orders: i64,

    #[serde(rename = "buyPrice")]
    pub buy_price: f64,

    #[serde(rename = "buyVolume")]
    buy_volume: i64,

    #[serde(rename = "buyMovingWeek")]
    buy_moving_week: i64,

    #[serde(rename = "buyOrders")]
    buy_orders: i64,
}
pub async fn get() -> Result<BazaarResponse> {
    let text = HTTP_CLIENT
        .get("https://api.hypixel.net/skyblock/bazaar")
        .send()
        .await
        .map_err(|x| anyhow!(x))?
        .body_string()
        .await
        .map_err(|x| anyhow!(x))?;
    Ok(serde_json::from_str(&text)?)
}
