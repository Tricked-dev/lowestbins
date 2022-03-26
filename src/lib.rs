pub mod bazaar;
pub mod fetch;
pub mod http_client;
pub mod nbt_utils;
pub mod server;
pub mod util;

use arc_swap::ArcSwap;
use std::{collections::HashMap, sync::Arc};

lazy_static::lazy_static! {
    static ref AUCTIONS: ArcSwap<HashMap<String, i64>> = ArcSwap::new(Arc::new(HashMap::new()));
}
