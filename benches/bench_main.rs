use std::io::{Cursor, Read};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use dashmap::DashMap;

use lowestbins::{
    fetch::{parse_hypixel, HypixelResponse},
    nbt_utils::Item,
};

fn parse_nbt_serde(data: &Vec<u8>) {
    let item: Item = serde_json::from_slice(data).unwrap();
    let nbt = &item.to_nbt().unwrap().i[0];
    let _pet = nbt.tag.extra_attributes.pet.clone().unwrap();
}
fn parse_nbt_simd(data: &Vec<u8>) {
    let mut data_mut = data.clone();
    let item: Item = simd_json::from_slice(&mut data_mut).unwrap();
    let nbt = &item.to_nbt().unwrap().i[0];
    let _pet = nbt.tag.extra_attributes.pet.clone().unwrap();
}

fn compare_nbt_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("NBT Parsing");
    let  data = r#"{"uuid":"e224c3087b0d4306b6a2879bd897336c","auctioneer":"5b005c6c9f214c4b8d2811551eab0467","profile_id":"d2f3941e6c73476b8a7608ad2b0391ae","coop":["a129d79b86e446579754efd35300e264","5b005c6c9f214c4b8d2811551eab0467","87a2f71b58344a73acc53d664d9c2d6e"],"start":1664445458945,"end":1664618258945,"item_name":"[Lvl 100] Dolphin","item_lore":"§8Fishing Pet\n\n§7Intelligence: §a+100\n§7Sea Creature Chance: §c+10%\n\n§6Pod Tactics\n§7§7Grants §b+10☂ Fishing Speed\n§b§7for each player within §a30\n§a§7blocks, up to §a5 §7players.\n\n§6Echolocation\n§7§7Grants §3+10α Sea Creature\n§3Chance§7.\n\n§6Splash Surprise\n§7§7Stun sea creatures for §a5s\n§a§7after fishing them up.\n\n§6Held Item: §6Washed-up Souvenir\n§7§7Grants §3+5α Sea Creature\n§3Chance§7.\n\n§b§lMAX LEVEL\n\n§7§eRight-click to add this pet to\n§eyour pet menu!\n\n§6§lLEGENDARY","extra":"[Lvl 100] Dolphin Skull Item","category":"misc","tier":"LEGENDARY","starting_bid":28000000,"item_bytes":"H4sIAAAAAAAAAI1Uy47jRBStdM8wSSRowQYWI1FjhhXJYMd2Xrso8XTcSifpOJ20g9CoXC7H1fFLdrmnHcSGPXv+oLfwC/0BbPgDPgRx3Z0ZDWIBkmW5bp1z7rlH5aojVEMVXkcIVY7QEXcrP1fQ02GcR6JSR8eCbI9Rbcxd9jog2wxQf9VR3drlQTB7G7G0io5MF73Uup6ueh5rEtWTm2qrKzcJoXJTUSiV3bbSUjwNePM0TlgqOMtqqCrYrchTlj20rqKnKxLkDP3KijN5c+XL7tVZQAuzDeulJQcz8zrpmNGqcIZm2wxhfzxoT4reB1hdkLUe2OqZv4kucidcyRN1EbDxQqHh5Y3dWoWbpbubjQZ7e3mh2GujsNf27Wy5229O7WK6vJDtpanBtzYdBf50D7Vrez/dG7odboJNeK7MTs/C8/1Wt9emurm+5N6V0gP3dfTM5VkSkKKGnkzilFWh+Bx9cn/Xfc0zn0dbPGcCal/c38EMggUB37KIsj6+vyPfKLKMvoQdixE8TBkpU8FDnxwAFABfAxnk2vPYxUtCBacZegkUeE5TEokMcA7g/vzlJ/yupZUw5iIJNgDlxSlmhPq4dMlS/JYLAJXtVblUIoBxgpjusgbOEyzickuHV+eRkL0CByfgwKB+DDgieByV4v+woIKF33/DHw6CPoby4zCALFU+AxULVDMfW3mapDxj74QskUc4AzY9sDNcGi+tZOj5o0viCfDvHYYUPgvBcKn7FeiOWeBiU7CwDK69hh7MbcI8VpzfsIin6MW/HOv/afjkIcLgfHCFJ8bKmEDpMXu24FtfNGnA6a6MjLguGOIZTpiANfoUIEWcpw/rkEX5i0OIoDYxTo3paLCwq+jJlIQMfQ6S301uAgzH4fvS/SgOEpgRTteJcStSMhAi5U4uWFZFz0DRjLwY/fGDJIqESX1pNJvMx+ZUakjl+biBkkeCjDUkdptI/darbkfVNV3r6O12T1G0jtFpSPAnpkB97wXIPvzqpfJ7ug+RlokCbj2wxsbozeX8jTW7XBlTcwEESiK3uMyYK/XlhpTnHD4kTVN1h7Z7TafruE1NY3qT0K7S7DgedZQeVTxFPjR7yHBYRnho+WO1vIbQ8dxYQjalINwv/0OvimqChywTJEzQSe/bFjwtrPf1Dh6cI3SEPhqRkGwZOkbob3Nyjtz2BAAA","claimed":false,"claimed_bidders":[],"highest_bid_amount":0,"last_updated":1664445458945,"bin":true,"bids":[],"item_uuid":"4435bc69b8bd44e5ac817bfcb19c1f10"}"#.as_bytes().to_vec();

    group.bench_with_input("Serde", &data.clone(), |b, i| b.iter(|| parse_nbt_serde(i)));
    group.bench_with_input("Simd", &data.clone(), |b, i| b.iter(|| parse_nbt_simd(i)));
}

fn parse_hypixel_auctions(c: &mut Criterion) {
    let mut data = include_bytes!("../resources/bench-auctions.bin").to_vec();
    let reader = Cursor::new(&mut data);
    let mut gz = flate2::read::GzDecoder::new(reader);
    let mut s = String::new();
    gz.read_to_string(&mut s).unwrap();
    let items = s
        .split("\n")
        .map(|x| serde_json::from_str::<HypixelResponse>(x).unwrap())
        .collect::<Vec<_>>();
    c.bench_function("Parsing Hypixel Responses", move |b| {
        b.iter(|| {
            let auctions: DashMap<String, u64> = DashMap::new();
            for item in items.iter() {
                parse_hypixel(item.auctions.clone(), &auctions);
            }
        })
    });
}
criterion_group! {
     benches, parse_hypixel_auctions,compare_nbt_parsing
}
criterion_main!(benches);
