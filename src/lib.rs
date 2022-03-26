pub mod bazaar;
pub mod fetch;
pub mod nbt_utils;
pub mod server;
pub mod util;

use std::{sync::Mutex, time::Duration};

use surf::{Client, Config};

lazy_static::lazy_static! {
   pub static ref AUCTIONS: Mutex<String> = Mutex::new(String::default());
   pub static ref HTTP_CLIENT: Client = Config::new()
        .set_timeout(Some(Duration::from_secs(50)))
        .try_into()
        .unwrap();
}
