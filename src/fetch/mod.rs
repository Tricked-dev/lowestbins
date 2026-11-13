use crate::{
    AUCTIONS, CONFIG,
    error::Result,
    fetch::{
        auctions::{get_auctions, get_auctions_page, parse_auctions},
        bazaar::get_bazaar_products,
    },
    set_last_updates,
    webhook::*,
};
use std::collections::BTreeMap;

use dashmap::DashMap;
use futures_util::{StreamExt, stream::FuturesUnordered};

use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub mod auctions;
pub mod bazaar;
pub mod util;

pub async fn fetch_auctions() -> Result<()> {
    let start = Instant::now();
    let hs = get_auctions_page(0).await?;

    let auctions: DashMap<String, u64> = DashMap::new();
    let bazaar: DashMap<String, f64> = DashMap::new();
    let instant_buy: DashMap<String, f64> = DashMap::new();
    let instant_sell: DashMap<String, f64> = DashMap::new();

    parse_auctions(hs.auctions, &auctions)?;

    let n = Instant::now();

    let auction_pages = async {
        let futures = FuturesUnordered::new();

        for page in 1..hs.total_pages {
            futures.push(get_auctions(page, &auctions));
        }

        let results: Vec<_> = futures.collect().await;
        for result in results {
            result?;
        }

        Ok::<(), anyhow::Error>(())
    };

    let (auction_result, bazaar_result) = tokio::join!(
        auction_pages,
        get_bazaar_products(&bazaar, &instant_buy, &instant_sell,)
    );

    auction_result?;

    let bazaar_timestamp = match bazaar_result {
        Ok(source_time) => {
            let timestamp = source_time.unwrap_or_else(|| {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64
            });
            Some(timestamp)
        }
        Err(error) => {
            tracing::warn!("Bazaar refresh failed; preserving previous prices: {error:?}");

            None
        }
    };

    let fetched = auctions.len() + bazaar.len();
    let fetch_time = n.elapsed();

    let mut new_auctions = BTreeMap::new();
    for kv in auctions.into_iter() {
        new_auctions.insert(kv.0, kv.1);
    }
    new_auctions.extend(CONFIG.overwrites.clone());

    let new_bazaar: BTreeMap<String, f64> = bazaar.into_iter().collect();

    let new_instant_buy: BTreeMap<String, f64> = instant_buy.into_iter().collect();

    let new_instant_sell: BTreeMap<String, f64> = instant_sell.into_iter().collect();

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

        if let Some(timestamp) = bazaar_timestamp {
            for id in new_bazaar.keys() {
                auc.historical_auctions.remove(id);
            }

            auc.bazaar = new_bazaar;
            auc.bazaar_instant_buy = new_instant_buy;
            auc.bazaar_instant_sell = new_instant_sell;
            auc.bazaar_updated_at = Some(timestamp);
        }

        auc.available_auctions = new_auctions.clone();
        auc.historical_auctions.extend(new_auctions);

        set_last_updates();

        auc.build_combined_map(crate::FilterType::All, crate::FilterPrice::Available)
    };

    crate::history::update_history(snapshot);
    crate::server::clear_response_cache();

    Ok(())
}
