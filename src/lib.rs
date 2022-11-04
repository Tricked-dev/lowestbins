#![doc = include_str!("../README.md")]
#![feature(test)]

pub mod error;
pub mod fetch;
pub mod nbt_utils;
pub mod server;
pub mod webhook;

const UA: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("CARGO_PKG_REPOSITORY"),
    ")"
);
pub const SOURCE: &str = "https://github.com/Tricked-dev/lowestbins";
pub const SPONSOR: &str = "https://github.com/sponsors/Tricked-dev";

use std::{
    collections::{BTreeMap, HashMap},
    env, fs,
    time::Instant,
};

use isahc::{config::DnsCache, prelude::Configurable, HttpClient};
use once_cell::sync::Lazy;
use parking_lot::Mutex;

const UPDATE_SECONDS: &str = "UPDATE_SECONDS";
const SAVE_TO_DISK: &str = "SAVE_TO_DISK";
const OVERWRITES: &str = "OVERWRITES";
const WEBHOOK_URL: &str = "WEBHOOK_URL";
const PORT: &str = "PORT";
const HOST: &str = "HOST";
const API_URL_ENV: &str = "API_URL";

#[derive(Debug)]
pub struct Conf {
    pub webhook_url: Option<String>,
    pub overwrites: HashMap<String, u64>,
    pub host: String,
    pub port: u16,
    pub update_seconds: u64,
    pub save_to_disk: bool,
}

impl Conf {
    fn init() -> Self {
        let host = env::var(HOST).unwrap_or_else(|_| "127.0.0.1".to_owned());
        let port = env::var(PORT).unwrap_or_else(|_| "8080".to_string());
        let save_to_disk = env::var(SAVE_TO_DISK).unwrap_or_else(|_| "1".to_owned());
        let update_seconds =
            env::var(UPDATE_SECONDS).map_or(60, |f| f.parse().expect("Invalid number for update_seconds"));
        Self {
            webhook_url: env::var(WEBHOOK_URL).ok(),
            overwrites: Conf::get_overwrites(),
            host,
            port: port.parse().expect("Invalid port"),
            save_to_disk: save_to_disk != "0",
            update_seconds,
        }
    }
    fn get_overwrites() -> HashMap<String, u64> {
        let overwrites = env::var(OVERWRITES).unwrap_or_default();
        let mut map = HashMap::new();
        for overwrite in overwrites.split(',') {
            let mut split = overwrite.split(':');
            let key = split.next().unwrap();
            if let Some(value) = split.next() {
                map.insert(key.to_string(), value.parse().unwrap());
            }
        }
        map
    }
}

// Using lazy it's considered better than lazy_static!

pub static API_URL: Lazy<String> =
    Lazy::new(|| env::var(API_URL_ENV).unwrap_or_else(|_| "https://api.hypixel.net".to_owned()));
pub static CONFIG: Lazy<Conf> = Lazy::new(Conf::init);
pub static HTTP_CLIENT: Lazy<HttpClient> = Lazy::new(|| {
    HttpClient::builder()
        .default_header("user-agent", UA)
        .max_connections_per_host(50)
        .tcp_nodelay()
        .dns_cache(DnsCache::Forever)
        .build()
        .unwrap()
});
pub static LAST_UPDATED: Lazy<Mutex<Instant>> = Lazy::new(|| Mutex::new(Instant::now()));

pub fn set_last_updates() {
    *LAST_UPDATED.lock() = Instant::now();
}
pub fn calc_next_update() -> u64 {
    let last_updated = LAST_UPDATED.lock();
    let elapsed = last_updated.elapsed().as_secs();
    if elapsed > CONFIG.update_seconds {
        0
    } else {
        CONFIG.update_seconds - elapsed
    }
}
// Honestly there should be a better way to do this in a more memory efficient way i think?
pub static AUCTIONS: Lazy<Mutex<BTreeMap<String, u64>>> = Lazy::new(|| {
    let mut res: BTreeMap<String, u64> = fs::read("auctions.json")
        .map(|x| serde_json::from_slice(&x).unwrap())
        .unwrap_or_default();
    let defaults = include_bytes!(concat!(env!("OUT_DIR"), "/sellprices.bin"));
    let map = rmp_serde::from_slice::<BTreeMap<String, u64>>(defaults).unwrap();
    res.extend(map);
    Mutex::new(res)
});
