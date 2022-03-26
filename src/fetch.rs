use crate::bazaar::get as get_bazaar;
use crate::util::{get, parse_hypixel};
use crate::AUCTIONS;
use futures::{stream, StreamExt};
use log::{debug, info};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub async fn fetch_auctions() {
    info!("fetching auctions");
    let started = Instant::now();

    let r = get(1).await;
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

                parse_hypixel(res.auctions, auctions)
            }
        })
        .buffer_unordered(200);

    println!("Total fetch time {}", nower.elapsed().as_millis());
    bodies
        .for_each(|res: HashMap<String, i64>| async {
            let auction_clone = auctions.clone();
            let handle = async move {
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
            handle.await;
        })
        .await;
    // bodies.for_each(|res| async {}).await;
    info!("fetching bazaar");
    let r = get_bazaar().await;
    let prods = r.products;
    for (key, val) in prods.iter() {
        auctions
            .clone()
            .lock()
            .unwrap()
            .insert(key.to_string(), val.quick_status.buy_price.round() as i64);
    }

    // let xs = serde_json::to_string(&*auctions.lock().unwrap()).unwrap();
    // debug!("writing file");
    println!("!! Total time taken {}", started.elapsed().as_secs());
    info!("!! Total time taken {}", started.elapsed().as_secs());
    AUCTIONS.store(Arc::new(auctions.lock().unwrap().clone()));
    // fs::write("lowestbins.json", xs).unwrap();
}
