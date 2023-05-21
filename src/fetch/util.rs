use isahc::AsyncReadResponseExt;
use serde::de::DeserializeOwned;

use crate::{error, API_URL, HTTP_CLIENT};

pub async fn get_path<T: DeserializeOwned>(path: &'_ str) -> error::Result<T> {
    #[allow(unused_mut)]
    let mut text = HTTP_CLIENT
        .get_async(format!("{API_URL}/skyblock/{path}", API_URL = *API_URL))
        .await?
        .bytes()
        .await?;

    Ok(serde_json::from_slice(&text)?)
}
