use crate::{HTTP_CLIENT, WEBHOOK_URL};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Embed {
    title: String,
    description: String,
}
#[derive(Serialize, Deserialize)]
pub struct Message {
    content: String,
    embeds: Vec<Embed>,
}
impl Message {
    pub fn new(content: String, embeds: Vec<Embed>) -> Self {
        Self { content, embeds }
    }
}
impl Embed {
    pub fn new(title: String, description: String) -> Self {
        Self { title, description }
    }
}
pub async fn send_embed(msg: Message) -> Result<()> {
    if let Some(webhook) = &*WEBHOOK_URL {
        HTTP_CLIENT
            .post(webhook)
            .body_json(&msg)
            .map_err(|x| anyhow!(x))?
            .send()
            .await
            .map_err(|x| anyhow!(x))?;
    }

    Ok(())
}
pub async fn send_webhook_text(msg: &str) -> Result<()> {
    send_embed(Message::new(msg.to_owned(), vec![])).await
}
