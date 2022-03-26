use crate::bazaar::get as get_bazaar;
use crate::util::{get, parse_hypixel};
use crate::AUCTIONS;
use anyhow::Result;
use futures::{stream, StreamExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub async fn fetch_auctions() -> Result<()> {
    let started = Instant::now();

    let r = get(1).await?;
    let auctions: Arc<Mutex<HashMap<String, i64>>> =
        Arc::new(Mutex::new(parse_hypixel(r.auctions, HashMap::new())));

    let mut pages: Vec<i64> = vec![];
    for a in 2..r.total_pages {
        pages.push(a);
    }

    let nower = Instant::now();
    let bodies = stream::iter(pages)
        .map(|url| {
            let client = &get;
            async move {
                let nows = Instant::now();
                let auctions: HashMap<String, i64> = HashMap::new();
                let res = client(url).await;
                println!("request time {}", nows.elapsed().as_millis());
                match res {
                    Ok(res) => Some(parse_hypixel(res.auctions, auctions)),
                    Err(e) => {
                        eprintln!("{e:?}");
                        None
                    }
                }
            }
        })
        .buffer_unordered(200);

    println!("Total fetch time {}", nower.elapsed().as_millis());
    bodies
        .for_each(|res: Option<HashMap<String, i64>>| async {
            //HashMap<String, i64>
            if let Some(res) = res {
                let auction_clone = auctions.clone();

                let mut auctions = auction_clone.lock().unwrap();
                for (x, y) in res.iter() {
                    match auctions.get(x) {
                        Some(s) => {
                            if s > y {
                                auctions.insert(x.clone(), *y);
                            };
                        }
                        None => {
                            auctions.insert(x.clone(), *y);
                        }
                    }
                }
                drop(auctions);
            };
        })
        .await;
    let r = get_bazaar().await?;
    let prods = r.products;
    for (key, val) in prods.iter() {
        auctions
            .clone()
            .lock()
            .unwrap()
            .insert(key.to_string(), val.quick_status.buy_price.round() as i64);
    }

    let xs = serde_json::to_string(&*auctions.lock().unwrap())?;
    drop(auctions);
    drop(prods);

    println!("!! Total time taken {}", started.elapsed().as_secs());
    let mut auc = AUCTIONS.lock().unwrap();
    *auc = xs;

    Ok(())
}
