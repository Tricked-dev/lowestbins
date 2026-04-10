#![doc = include_str!("../README.md")]

pub mod error;
pub mod fetch;
pub mod nbt_utils;
pub mod server;
pub mod store;
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

use std::{collections::HashMap, env};

use once_cell::sync::Lazy;
use reqwest::Client;

use crate::store::Store;

const UPDATE_SECONDS: &str = "UPDATE_SECONDS";
const ENABLE_HISTORY: &str = "ENABLE_HISTORY";
const OVERWRITES: &str = "OVERWRITES";
const WEBHOOK_URL: &str = "WEBHOOK_URL";
const PORT: &str = "PORT";
const HOST: &str = "HOST";
const API_URL_ENV: &str = "API_URL";
const REDIS_URL_ENV: &str = "REDIS_URL";

#[derive(Debug)]
pub struct Conf {
    pub webhook_url: Option<String>,
    pub overwrites: HashMap<String, u64>,
    pub host: String,
    pub port: u16,
    pub update_seconds: u64,
    pub enable_history: bool,
    pub redis_url: Option<String>,
}

impl Conf {
    fn init() -> Self {
        let host = env::var(HOST).unwrap_or_else(|_| "127.0.0.1".to_owned());
        let port = env::var(PORT).unwrap_or_else(|_| "8080".to_string());
        let update_seconds =
            env::var(UPDATE_SECONDS).map_or(60, |f| f.parse().expect("Invalid number for update_seconds"));
        let enable_history = env::var(ENABLE_HISTORY).unwrap_or_else(|_| "0".to_owned());
        Self {
            webhook_url: env::var(WEBHOOK_URL).ok(),
            overwrites: Conf::get_overwrites(),
            host,
            port: port.parse().expect("Invalid port"),
            enable_history: enable_history != "0",
            update_seconds,
            redis_url: env::var(REDIS_URL_ENV).ok(),
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

pub static API_URL: Lazy<String> =
    Lazy::new(|| env::var(API_URL_ENV).unwrap_or_else(|_| "https://api.hypixel.net".to_owned()));
pub static CONFIG: Lazy<Conf> = Lazy::new(Conf::init);

pub static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .user_agent(UA)
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
});

pub static STORE: Lazy<Store> = Lazy::new(|| {
    // Store initialization is sync for Lazy. RedisStore needs async init,
    // so we create a temporary runtime for the connection.
    if let Some(url) = &CONFIG.redis_url {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build()
            .expect("Failed to create runtime for Redis connection");
        let redis_store = rt
            .block_on(store::RedisStore::new(url))
            .expect("Failed to connect to Redis");
        Store::Redis(redis_store)
    } else {
        tracing::info!("No REDIS_URL set, using in-memory store");
        Store::Memory(store::MemoryStore::new())
    }
});

include!(concat!(env!("OUT_DIR"), "/prices_map.rs"));

pub async fn calc_next_update() -> u64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let last = STORE.get_last_updated().await.unwrap_or(0);
    if last == 0 {
        return CONFIG.update_seconds;
    }
    let elapsed = now.saturating_sub(last);
    CONFIG.update_seconds.saturating_sub(elapsed)
}

pub fn round_to_nearest_15(value: u64) -> u64 {
    ((value + 7) / 15 * 15).max(15)
}