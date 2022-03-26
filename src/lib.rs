pub mod bazaar;
pub mod fetch;
pub mod http_client;
pub mod nbt_utils;
pub mod server;
pub mod util;

use arc_swap::ArcSwap;
use std::sync::Arc;

lazy_static::lazy_static! {
    static ref AUCTIONS: ArcSwap<String> = ArcSwap::new(Arc::new(String::new()));
}
