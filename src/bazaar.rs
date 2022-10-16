use std::collections::HashMap;

use crate::error::Result;
use isahc::AsyncReadResponseExt;
use serde::Deserialize;

use crate::{API_URL, HTTP_CLIENT};

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
pub async fn get() -> Result<BazaarResponse> {
    #[allow(unused_mut)]
    let mut text = HTTP_CLIENT
        .get_async(format!("{API_URL}/skyblock/bazaar", API_URL = *API_URL))
        .await?
        .bytes()
        .await?;

    #[cfg(feature = "simd")]
    return Ok(simd_json::from_slice(&mut text)?);
    #[cfg(not(feature = "simd"))]
    return Ok(serde_json::from_slice(&text)?);
}
