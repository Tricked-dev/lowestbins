use std::{
    collections::{BTreeMap, HashMap},
    io::{Cursor, Read},
};

use dashmap::DashMap;
use lowestbins::fetch::auctions::{parse_auctions, HypixelResponse};

// Regenerate the bench-auctions.json fixture against the current parser by
// running:
//
//     LOWESTBINS_REGEN_FIXTURE=1 cargo test --test test_data parsing_works
//
// without the env var, the test asserts the parser output matches the
// checked-in fixture.
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

    if std::env::var("LOWESTBINS_REGEN_FIXTURE").is_ok() {
        // BTreeMap so the on-disk fixture is sorted and stable across runs.
        let sorted: BTreeMap<&String, &u64> = r.iter().collect();
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/resources/bench-auctions.json");
        std::fs::write(path, serde_json::to_vec_pretty(&sorted).unwrap()).unwrap();
        eprintln!("wrote fresh fixture to {path} ({} entries)", sorted.len());
        return;
    }

    assert_eq!(
        r,
        serde_json::from_slice::<HashMap<String, u64>>(include_bytes!("../resources/bench-auctions.json")).unwrap()
    );
}
