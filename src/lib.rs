pub mod bazaar;
pub mod fetch;
pub mod nbt_utils;
pub mod server;

use std::env;
use std::{sync::Mutex, time::Duration};

use surf::{Client, Config};

lazy_static::lazy_static! {
   pub static ref AUCTIONS: Mutex<Vec<u8>> = Mutex::new(Vec::new());
   pub static ref HTTP_CLIENT: Client = Config::new()
        .set_timeout(Some(Duration::from_secs(50)))
        .try_into()
        .unwrap();
   pub static ref PARRALEL :usize = env::var("PARRALEL").map_or(32, |f|f.parse().unwrap());
}
