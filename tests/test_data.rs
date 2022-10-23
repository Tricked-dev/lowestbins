use std::{
    collections::HashMap,
    io::{Cursor, Read},
};

use dashmap::DashMap;
use lowestbins::fetch::auctions::{parse_auctions, HypixelResponse};

#[test]
fn parsing_works() {
    let mut data = include_bytes!("../resources/bench-auctions.bin").to_vec();
    let reader = Cursor::new(&mut data);
    let mut gz = flate2::read::GzDecoder::new(reader);
    let mut s = String::new();
    gz.read_to_string(&mut s).unwrap();
    let items = s
        .lines()
        .map(|x| serde_json::from_str::<HypixelResponse>(x).unwrap())
        .collect::<Vec<_>>();

    let auctions: DashMap<String, u64> = DashMap::new();
    for item in items.iter() {
        parse_auctions(item.auctions.clone(), &auctions).unwrap();
    }
    let mut r = HashMap::new();
    r.extend(auctions);
    assert_eq!(
        r,
        serde_json::from_slice::<HashMap<String, u64>>(include_bytes!("../resources/bench-auctions.json")).unwrap()
    );
}
