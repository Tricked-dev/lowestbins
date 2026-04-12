use crate::{
    error::Result,
    fetch::{
        auctions::{get_auctions, get_auctions_page, parse_auctions},
        bazaar::get_bazaar_products,
    },
    set_last_updates,
    webhook::*,
    AUCTIONS, CONFIG,
};
use std::collections::BTreeMap;

use dashmap::DashMap;
use futures_util::{stream::FuturesUnordered, FutureExt, StreamExt};

use std::time::Instant;

pub mod auctions;
pub mod bazaar;
pub mod util;

pub async fn fetch_auctions() -> Result<()> {
    let start = Instant::now();
    let hs = get_auctions_page(0).await?;

    let auctions: DashMap<String, u64> = DashMap::new();
    let bazaar: DashMap<String, f64> = DashMap::new();
    parse_auctions(hs.auctions, &auctions)?;

    let futures = FuturesUnordered::new();
    let n = Instant::now();
    for url in 1..hs.total_pages {
        futures.push(get_auctions(url, &auctions).boxed());
    }
    futures.push(get_bazaar_products(&bazaar).boxed());

    let _: Vec<_> = futures.collect().await;
    let fetched = auctions.len() + bazaar.len();
    let fetch_time = n.elapsed();

    let mut new_auctions = BTreeMap::new();
    for kv in auctions.into_iter() {
        new_auctions.insert(kv.0, kv.1);
    }
    new_auctions.extend(CONFIG.overwrites.clone());

    let mut new_bazaar = BTreeMap::new();
    for kv in bazaar.into_iter() {
        new_bazaar.insert(kv.0, kv.1);
    }

    tracing::debug!("Fetched {} items in {:?}", fetched, fetch_time);
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

    let snapshot = {
        let mut auc = AUCTIONS.lock();
        for k in new_bazaar.keys() {
            auc.historical_auctions.remove(k);
        }
        auc.available_auctions = new_auctions.clone();
        auc.historical_auctions.extend(new_auctions);
        auc.bazaar = new_bazaar;

        set_last_updates();

        auc.build_combined_map(crate::FilterType::All, crate::FilterPrice::Available)
    };
    crate::history::update_history(snapshot);
    crate::server::clear_response_cache();

    Ok(())
}
