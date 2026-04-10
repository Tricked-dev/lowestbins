pub mod memory;
pub mod redis_store;

use std::collections::HashMap;

use crate::error::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Auction,
    Bazaar,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PriceMode {
    Available,
    #[default]
    Historical,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SourceFilter {
    Auction,
    Bazaar,
    #[default]
    All,
}

pub use memory::MemoryStore;
pub use redis_store::RedisStore;

#[allow(clippy::large_enum_variant)]
pub enum Store {
    Redis(RedisStore),
    Memory(MemoryStore),
}

impl std::fmt::Display for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Store::Redis(_) => write!(f, "Redis"),
            Store::Memory(_) => write!(f, "Memory"),
        }
    }
}

impl Store {
    pub async fn write_cycle(
        &self,
        auction_prices: HashMap<String, u64>,
        bazaar_prices: HashMap<String, u64>,
    ) -> Result<()> {
        match self {
            Store::Redis(s) => s.write_cycle(auction_prices, bazaar_prices).await,
            Store::Memory(s) => s.write_cycle(auction_prices, bazaar_prices).await,
        }
    }

    pub async fn get_prices(&self, source: Source) -> Result<HashMap<String, u64>> {
        match self {
            Store::Redis(s) => s.get_prices(source).await,
            Store::Memory(s) => s.get_prices(source).await,
        }
    }

    pub async fn get_last_known(&self, source: Source) -> Result<HashMap<String, u64>> {
        match self {
            Store::Redis(s) => s.get_last_known(source).await,
            Store::Memory(s) => s.get_last_known(source).await,
        }
    }

    pub async fn get_averages(&self, source: Source, days: u8) -> Result<HashMap<String, u64>> {
        match self {
            Store::Redis(s) => s.get_averages(source, days).await,
            Store::Memory(s) => s.get_averages(source, days).await,
        }
    }

    pub async fn get_last_updated(&self) -> Result<u64> {
        match self {
            Store::Redis(s) => s.get_last_updated().await,
            Store::Memory(s) => s.get_last_updated().await,
        }
    }

    pub async fn resolve_prices(
        &self,
        filter: SourceFilter,
        mode: PriceMode,
    ) -> Result<HashMap<String, u64>> {
        let get = |source: Source| async move {
            match mode {
                PriceMode::Available => self.get_prices(source).await,
                PriceMode::Historical => self.get_last_known(source).await,
            }
        };

        match filter {
            SourceFilter::Auction => get(Source::Auction).await,
            SourceFilter::Bazaar => get(Source::Bazaar).await,
            SourceFilter::All => {
                let (auction, bazaar) =
                    tokio::join!(get(Source::Auction), get(Source::Bazaar),);
                let mut merged = auction?;
                for (key, price) in bazaar? {
                    let entry = merged.entry(key).or_insert(u64::MAX);
                    if price < *entry {
                        *entry = price;
                    }
                }
                Ok(merged)
            }
        }
    }

    pub async fn resolve_averages(
        &self,
        filter: SourceFilter,
        days: u8,
    ) -> Result<HashMap<String, u64>> {
        match filter {
            SourceFilter::Auction => self.get_averages(Source::Auction, days).await,
            SourceFilter::Bazaar => self.get_averages(Source::Bazaar, days).await,
            SourceFilter::All => {
                let (auction, bazaar) = tokio::join!(
                    self.get_averages(Source::Auction, days),
                    self.get_averages(Source::Bazaar, days),
                );
                let mut merged = auction?;
                for (key, price) in bazaar? {
                    let entry = merged.entry(key).or_insert(u64::MAX);
                    if price < *entry {
                        *entry = price;
                    }
                }
                Ok(merged)
            }
        }
    }
}

pub fn parse_query_params(query: Option<&str>) -> (SourceFilter, PriceMode) {
    let mut source = SourceFilter::default();
    let mut price = PriceMode::default();

    if let Some(q) = query {
        for part in q.split('&') {
            let mut kv = part.splitn(2, '=');
            match (kv.next(), kv.next()) {
                (Some("type"), Some("auction")) => source = SourceFilter::Auction,
                (Some("type"), Some("bazaar")) => source = SourceFilter::Bazaar,
                (Some("type"), Some("all")) => source = SourceFilter::All,
                (Some("price"), Some("available")) => price = PriceMode::Available,
                (Some("price"), Some("historical")) => price = PriceMode::Historical,
                _ => {}
            }
        }
    }

    (source, price)
}
