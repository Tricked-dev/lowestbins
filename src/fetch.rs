use crate::bazaar::get as get_bazaar;
use crate::util::{get, parse_hypixel};
use std::collections::HashMap;
use std::fs;
use std::time::Instant;

pub async fn fetch_auctions() {
    let started = Instant::now();

    let mut auctions: HashMap<String, i64> = HashMap::new();

    let r = get(1).await;
    auctions = parse_hypixel(r.auctions, auctions);
    for a in 2..r.total_pages {
        println!("------------------------ req: {}", a);
        let now = Instant::now();
        let r = get(a).await;
        println!(": request took {} miliseconds", now.elapsed().as_millis());
        let nowss = Instant::now();
        auctions = parse_hypixel(r.auctions, auctions);
        println!("$ parsing took {} miliseconds", nowss.elapsed().as_millis());
        println!(
            "! request and parsing took {} miliseconds",
            now.elapsed().as_millis()
        );
    }
    let r = get_bazaar().await;
    let prods = r.products;
    for (key, val) in prods.iter() {
        auctions.insert(key.to_string(), val.quick_status.buy_price.round() as i64);
    }

    let xs = serde_json::to_string(&auctions).unwrap();
    println!("writing file");
    println!("!! Total time taken {}", started.elapsed().as_secs());
    fs::write("lowestbins.json", xs).unwrap();
}
