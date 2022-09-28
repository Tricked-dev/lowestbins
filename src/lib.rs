pub mod bazaar;
pub mod fetch;
pub mod nbt_utils;
pub mod server;
pub mod webhook;

use std::{collections::HashMap, env, fs, sync::Mutex, time::Duration};

use surf::{Client, Config};

static UPDATE_SECONDS: &str = "UPDATE_SECONDS";
static SAVE_TO_DISK: &str = "SAVE_TO_DISK";
static OVERWRITES: &str = "OVERWRITES";
static WEBHOOK_URL: &str = "WEBHOOK_URL";
static PORT: &str = "PORT";
static HOST: &str = "HOST";

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
   pub static ref HTTP_CLIENT: Client = Config::new()
        .set_timeout(Some(Duration::from_secs(50)))
        .try_into()
        .unwrap();
   pub static ref CONFIG: Conf = Conf::init();
}
