use crate::bazaar::get as get_bazaar;
use crate::util::{get, parse_hypixel};
use futures::future;
use log::{debug, info};
use std::collections::HashMap;
use std::fs;
use std::time::Instant;

pub async fn fetch_auctions() {
    info!("fetching auctions");
    let started = Instant::now();
    let mut auctions: HashMap<String, i64> = HashMap::new();

    let r = get(1).await;
    auctions = parse_hypixel(r.auctions, auctions);
    let mut pages: Vec<i64> = vec![];
    for a in 2..r.total_pages {
        pages.push(a);
    }

    let nower = Instant::now();
    let bodies = future::join_all(pages.into_iter().map(|url| {
        let client = &get;
        async move {
            let nows = Instant::now();
            let auctions: HashMap<String, i64> = HashMap::new();
            let res = client(url).await;
            println!("request time {}", nows.elapsed().as_millis());

            parse_hypixel(res.auctions, auctions)
        }
    }))
    .await;
    println!("Total fetch time {}", nower.elapsed().as_millis());
    for body in bodies {
        for (x, y) in body.iter() {
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
    }

    info!("fetching bazaar");
    let r = get_bazaar().await;
    let prods = r.products;
    for (key, val) in prods.iter() {
        auctions.insert(key.to_string(), val.quick_status.buy_price.round() as i64);
    }

    let xs = serde_json::to_string(&auctions).unwrap();
    debug!("writing file");
    println!("!! Total time taken {}", started.elapsed().as_secs());
    info!("!! Total time taken {}", started.elapsed().as_secs());
    fs::write("lowestbins.json", xs).unwrap();
}
