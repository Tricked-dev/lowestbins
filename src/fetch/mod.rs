use crate::{
    error::Result,
    fetch::{
        auctions::{get_auctions_page, get_auctions_page_parsed, parse_auctions},
        bazaar::get_bazaar_products,
    },
    webhook::*,
    CONFIG, STORE,
};

use futures_util::{stream::FuturesUnordered, FutureExt, StreamExt};

use std::collections::HashMap;
use std::time::Instant;

pub mod auctions;
pub mod bazaar;
pub mod util;

enum FetchResult {
    Auction(HashMap<String, u64>),
    Bazaar(HashMap<String, u64>),
}

pub async fn fetch_auctions() -> Result<()> {
    let start = Instant::now();
    let hs = get_auctions_page(0).await?;

    let first_page = parse_auctions(hs.auctions)?;

    let futures: FuturesUnordered<_> = FuturesUnordered::new();
    let n = Instant::now();

    for page in 1..hs.total_pages {
        futures.push(
            async move {
                get_auctions_page_parsed(page)
                    .await
                    .map(FetchResult::Auction)
            }
            .boxed(),
        );
    }
    futures.push(
        async {
            get_bazaar_products()
                .await
                .map(FetchResult::Bazaar)
        }
        .boxed(),
    );

    let mut auction_prices = first_page;
    let mut bazaar_prices = HashMap::new();

    let results: Vec<_> = futures.collect().await;
    for result in results {
        match result {
            Ok(FetchResult::Bazaar(bz)) => {
                bazaar_prices = bz;
            }
            Ok(FetchResult::Auction(page)) => {
                for (key, price) in page {
                    let entry = auction_prices.entry(key).or_insert(u64::MAX);
                    if price < *entry {
                        *entry = price;
                    }
                }
            }
            Err(e) => {
                tracing::error!("Error fetching page: {e:?}");
            }
        }
    }

    // Seed bazaar with hardcoded sell prices (only if not already present from live data)
    for (key, price) in crate::get_prices_map() {
        bazaar_prices.entry(key).or_insert(price);
    }

    // Apply overwrites to auction prices
    for (key, price) in &CONFIG.overwrites {
        auction_prices.insert(key.clone(), *price);
    }

    let fetched = auction_prices.len() + bazaar_prices.len();
    let fetch_time = n.elapsed();

    tracing::debug!("Fetched {} items in {:?}", fetched, fetch_time);

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

    // Write to store
    let store = &*STORE;
    if let Err(e) = store.write_cycle(auction_prices, bazaar_prices).await {
        tracing::error!("Store write_cycle failed: {e:?}");
    }

    Ok(())
}
