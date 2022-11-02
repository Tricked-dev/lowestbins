use crate::{
    error::Result,
    fetch::{
        auctions::{get_auctions, get_auctions_page, parse_auctions},
        bazaar::get_bazaar_products,
    },
    set_last_updates,
    webhook::*,
    AUCTIONS, CONFIG, LAST_UPDATED,
};

use dashmap::DashMap;
use futures_util::{stream::FuturesUnordered, FutureExt, StreamExt};

use std::time::Instant;

pub mod auctions;
pub mod bazaar;
pub mod util;

pub async fn fetch_auctions() -> Result<()> {
    let start = std::time::Instant::now();
    let hs = get_auctions_page(0).await?;

    let auctions: DashMap<String, u64> = DashMap::new();
    parse_auctions(hs.auctions, &auctions)?;

    let futures = FuturesUnordered::new();
    let n = Instant::now();
    for url in 1..hs.total_pages {
        futures.push(get_auctions(url, &auctions).boxed());
    }
    futures.push(get_bazaar_products(&auctions).boxed());

    let _: Vec<_> = futures.collect().await;
    let fetched = auctions.len();
    let fetch_time = n.elapsed();

    let mut new_auctions = DashMap::new();
    new_auctions.extend(auctions.clone());
    drop(auctions);
    new_auctions.extend(CONFIG.overwrites.clone());

    tracing::debug!("Fetched {} auctions in {:?}", fetched, fetch_time);
    // It only sends if the WEBHOOK_URL env var is set
    send_embed(Message::new(
        "Auctions updated".to_owned(),
        vec![Embed::new(
            "Auctions updated".to_owned(),
            format!(
                "Fetched: {} items\nFetch Time: {:?}\nTime: {:?}",
                fetched,
                fetch_time,
                start.elapsed()
            ),
        )],
    ))
    .await?;

    let mut auc = AUCTIONS.lock().expect("Failed to lock auctions");
    auc.extend(new_auctions);
    set_last_updates();
    Ok(())
}
