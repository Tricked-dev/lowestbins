use std::time::Duration;

use lazy_static::lazy_static;

use surf::{Client, Config};
lazy_static! {
    pub static ref HTTP_CLIENT: Client = Config::new()
        .set_timeout(Some(Duration::from_secs(20)))
        .try_into()
        .unwrap();
}
