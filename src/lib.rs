pub mod bazaar;
pub mod fetch;
pub mod nbt_utils;
pub mod server;
pub mod webhook;

use std::{collections::HashMap, env, fs, sync::Mutex, time::Duration};

use isahc::{
    config::{NetworkInterface, VersionNegotiation},
    prelude::*,
    HttpClient,
};

const UPDATE_SECONDS: &str = "UPDATE_SECONDS";
const SAVE_TO_DISK: &str = "SAVE_TO_DISK";
const OVERWRITES: &str = "OVERWRITES";
const WEBHOOK_URL: &str = "WEBHOOK_URL";
const PORT: &str = "PORT";
const HOST: &str = "HOST";

#[cfg(feature = "local")]
const API_UR: &str = "http://0.0.0.0:8000";
#[cfg(not(feature = "local"))]
const API_UR: &str = "https://api.hypixel.net";

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
        let update_seconds = env::var(UPDATE_SECONDS).map_or(60, |f| f.parse().unwrap());
        Self {
            webhook_url: env::var(WEBHOOK_URL).ok(),
            overwrites: Conf::get_overwrites(),
            host,
            port: port.parse().unwrap(),
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

lazy_static::lazy_static! {
   pub static ref AUCTIONS: Mutex<HashMap<String, u64>> ={
      let defaults = include_bytes!(concat!(env!("OUT_DIR"), "/sellprices.json"));
      let mut res: HashMap<String, u64> = fs::read("auctions.json")
            .map(|x| serde_json::from_slice(&x).unwrap())
            .unwrap_or_default();
      let map = serde_json::from_slice::<HashMap<String, f64>>(defaults).unwrap();
      res.extend(map.into_iter().map(|(k, v)| (k, v.round() as u64)));
      Mutex::new(res)
   };
   pub static ref HTTP_CLIENT: HttpClient = HttpClient::builder()
        .default_header("user-agent", "Lowestbins/1.3.0")
        .build()
        .unwrap();
   pub static ref CONFIG: Conf = Conf::init();
}
