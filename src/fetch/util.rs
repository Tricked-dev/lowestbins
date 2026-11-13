use serde::de::DeserializeOwned;

use crate::{API_URL, HTTP_CLIENT, error};

pub async fn get_path<T: DeserializeOwned>(path: &'_ str) -> error::Result<T> {
    #[allow(unused_mut)]
    let mut text = HTTP_CLIENT
        .get(format!("{API_URL}/v2/skyblock/{path}", API_URL = *API_URL))
        .send()
        .await?
        .bytes()
        .await?;

    Ok(serde_json::from_slice(&text)?)
}
