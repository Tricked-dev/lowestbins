#![doc = include_str!("../README.md")]

pub mod error;
pub mod fetch;
pub mod history;
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

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use reqwest::Client;

const UPDATE_SECONDS: &str = "UPDATE_SECONDS";
const SAVE_TO_DISK: &str = "SAVE_TO_DISK";
const ENABLE_HISTORY: &str = "ENABLE_HISTORY";
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
    pub enable_history: bool,
}

impl Conf {
    fn init() -> Self {
        let host = env::var(HOST).unwrap_or_else(|_| "127.0.0.1".to_owned());
        let port = env::var(PORT).unwrap_or_else(|_| "8080".to_string());
        let save_to_disk = env::var(SAVE_TO_DISK).unwrap_or_else(|_| "0".to_owned());
        let update_seconds =
            env::var(UPDATE_SECONDS).map_or(60, |f| f.parse().expect("Invalid number for update_seconds"));
        let enable_history = env::var(ENABLE_HISTORY).unwrap_or_else(|_| "0".to_owned());
        Self {
            webhook_url: env::var(WEBHOOK_URL).ok(),
            overwrites: Conf::get_overwrites(),
            host,
            port: port.parse().expect("Invalid port"),
            save_to_disk: save_to_disk != "0",
            enable_history: enable_history != "0",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FilterType {
    All = 0,
    Auction = 1,
    Bazaar = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FilterPrice {
    Historical = 0,
    Available = 1,
}

#[derive(Default, Clone)]
pub struct PriceState {
    pub historical_auctions: BTreeMap<String, u64>,
    pub available_auctions: BTreeMap<String, u64>,
    pub bazaar: BTreeMap<String, f64>,
}

impl PriceState {
    pub fn get_price(&self, id: &str, q_type: FilterType, q_price: FilterPrice) -> Option<f64> {
        let show_bz = q_type == FilterType::Bazaar || q_type == FilterType::All;
        let show_auc = q_type == FilterType::Auction || q_type == FilterType::All;

        #[allow(clippy::collapsible_if)]
        if show_bz {
            if let Some(&p) = self.bazaar.get(id) {
                return Some(p);
            }
        }
        if show_auc {
            let target = if q_price == FilterPrice::Available {
                &self.available_auctions
            } else {
                &self.historical_auctions
            };
            if let Some(&p) = target.get(id) {
                return Some(p as f64);
            }
        }

        None
    }

    pub fn for_each_price<F>(&self, q_type: FilterType, q_price: FilterPrice, mut f: F)
    where
        F: FnMut(&str, f64),
    {
        let show_auc = q_type == FilterType::Auction || q_type == FilterType::All;
        let show_bz = q_type == FilterType::Bazaar || q_type == FilterType::All;

        if show_auc {
            let target = if q_price == FilterPrice::Available {
                &self.available_auctions
            } else {
                &self.historical_auctions
            };
            for (k, &v) in target {
                f(k, v as f64);
            }
        }
        if show_bz {
            for (k, &v) in &self.bazaar {
                f(k, v);
            }
        }
    }

    pub fn build_combined_map(&self, q_type: FilterType, q_price: FilterPrice) -> BTreeMap<String, f64> {
        let mut result = BTreeMap::new();
        self.for_each_price(q_type, q_price, |k, v| {
            result.insert(k.to_string(), v);
        });
        result
    }
}

// Using lazy it's considered better than lazy_static!

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

pub static LAST_UPDATED: Lazy<Mutex<Instant>> = Lazy::new(|| Mutex::new(Instant::now()));

pub fn set_last_updates() {
    *LAST_UPDATED.lock() = Instant::now();
}

pub fn round_to_nearest_15(value: u64) -> u64 {
    let remainder = value % 15;
    if value < 15 {
        15
    } else if remainder == 0 {
        value
    } else if remainder < 8 {
        value - remainder
    } else {
        value + (15 - remainder)
    }
}

pub fn calc_next_update() -> u64 {
    let last_updated = LAST_UPDATED.lock();
    let elapsed = last_updated.elapsed().as_secs();
    CONFIG.update_seconds.saturating_sub(elapsed)
}

include!(concat!(env!("OUT_DIR"), "/prices_map.rs"));

// Honestly there should be a better way to do this in a more memory efficient way i think?
pub static AUCTIONS: Lazy<Mutex<PriceState>> = Lazy::new(|| {
    let historical_auctions: BTreeMap<String, u64> = fs::read("auctions.json")
        .map(|x| serde_json::from_slice(&x).unwrap_or_default())
        .unwrap_or_default();

    let mut res = PriceState {
        historical_auctions,
        available_auctions: BTreeMap::new(),
        bazaar: BTreeMap::new(),
    };
    res.historical_auctions.extend(HashMap::from(get_prices_map()));
    Mutex::new(res)
});
