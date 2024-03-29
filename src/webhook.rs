use crate::{error::Result, CONFIG, HTTP_CLIENT};
// use isahc::{AsyncBody, Request};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Embed {
    title: String,
    description: String,
    color: i32,
}
#[derive(Serialize, Debug)]
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
        Self {
            title,
            description,
            color: 1377743,
        }
    }
}
pub async fn send_embed(msg: Message) -> Result<()> {
    if let Some(webhook) = &CONFIG.webhook_url {
        HTTP_CLIENT
            .post(webhook)
            .body(serde_json::to_string(&msg)?)
            .send()
            .await?;
    }

    Ok(())
}
pub async fn send_webhook_text(msg: &str) -> Result<()> {
    send_embed(Message::new(msg.to_owned(), vec![])).await
}
