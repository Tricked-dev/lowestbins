use std::collections::HashMap;

use anyhow::{anyhow, Result};
use isahc::AsyncReadResponseExt;
use serde::{Deserialize, Serialize};

use crate::HTTP_CLIENT;

#[derive(Serialize, Deserialize)]
pub struct BazaarResponse {
    #[serde(rename = "products")]
    pub products: HashMap<String, Product>,
}

#[derive(Serialize, Deserialize)]
pub struct Product {
    #[serde(rename = "product_id")]
    product_id: String,

    #[serde(rename = "quick_status")]
    pub quick_status: QuickStatus,
}

#[derive(Serialize, Deserialize)]
pub struct QuickStatus {
    #[serde(rename = "buyPrice")]
    pub buy_price: f64,
}
pub async fn get() -> Result<BazaarResponse> {
    let mut text = HTTP_CLIENT
        .get_async("https://api.hypixel.net/skyblock/bazaar")
        .await
        .map_err(|x| anyhow!(x))?
        .bytes()
        .await
        .map_err(|x| anyhow!(x))?;

    #[cfg(feature = "simd")]
    return Ok(simd_json::from_slice(&mut text)?);
    #[cfg(not(feature = "simd"))]
    return Ok(serde_json::from_slice(&text)?);
}
