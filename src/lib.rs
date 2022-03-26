pub mod bazaar;
pub mod fetch;
pub mod nbt_utils;
pub mod server;
pub mod util;

use std::{sync::Arc, time::Duration};

use arc_swap::ArcSwap;
use surf::{Client, Config};

lazy_static::lazy_static! {
   pub static ref AUCTIONS: ArcSwap<String> = ArcSwap::new(Arc::new(String::new()));

   pub static ref HTTP_CLIENT: Client = Config::new()
        .set_timeout(Some(Duration::from_secs(20)))
        .try_into()
        .unwrap();
}
