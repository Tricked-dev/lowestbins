use std::collections::HashMap;

use redis::aio::ConnectionManager;
use redis::{pipe, AsyncCommands};

use super::Source;
use crate::error::Result;

const DAY_SLOTS: usize = 7;

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn day_start_of(ts: u64) -> u64 {
    ts - (ts % 86400)
}

fn prices_key(source: Source) -> &'static str {
    match source {
        Source::Auction => "auction:prices",
        Source::Bazaar => "bazaar:prices",
    }
}

fn last_key(source: Source) -> &'static str {
    match source {
        Source::Auction => "auction:last",
        Source::Bazaar => "bazaar:last",
    }
}

fn acc_sums_key(source: Source) -> &'static str {
    match source {
        Source::Auction => "history:auction:acc:sums",
        Source::Bazaar => "history:bazaar:acc:sums",
    }
}

fn acc_counts_key(source: Source) -> &'static str {
    match source {
        Source::Auction => "history:auction:acc:counts",
        Source::Bazaar => "history:bazaar:acc:counts",
    }
}

fn day_sums_key(source: Source, day: usize) -> String {
    let prefix = match source {
        Source::Auction => "history:auction",
        Source::Bazaar => "history:bazaar",
    };
    format!("{prefix}:day:{day}:sums")
}

fn day_counts_key(source: Source, day: usize) -> String {
    let prefix = match source {
        Source::Auction => "history:auction",
        Source::Bazaar => "history:bazaar",
    };
    format!("{prefix}:day:{day}:counts")
}

pub struct RedisStore {
    conn: ConnectionManager,
}

impl RedisStore {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let conn = ConnectionManager::new(client).await?;
        Ok(Self { conn })
    }

    pub async fn write_cycle(
        &self,
        auction_prices: HashMap<String, u64>,
        bazaar_prices: HashMap<String, u64>,
    ) -> Result<()> {
        let mut conn = self.conn.clone();
        let now = now_secs();

        // Check for day rollover
        if crate::CONFIG.enable_history {
            let day_acc_start: Option<String> = conn.get("meta:day_acc_start").await?;
            let current_day = day_start_of(now);
            let stored_day = day_acc_start
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);

            if stored_day != 0 && day_start_of(stored_day) != current_day {
                self.rollover_day(&mut conn, current_day).await?;
            }

            if stored_day == 0 {
                let _: () = conn
                    .set("meta:day_acc_start", current_day.to_string())
                    .await?;
            }
        }

        let mut p = pipe();
        p.atomic();

        // Current prices: wipe and rewrite
        for (source, prices) in [
            (Source::Auction, &auction_prices),
            (Source::Bazaar, &bazaar_prices),
        ] {
            let key = prices_key(source);
            p.del(key);
            if !prices.is_empty() {
                let items: Vec<(String, String)> = prices
                    .iter()
                    .map(|(k, v)| (k.clone(), v.to_string()))
                    .collect();
                p.hset_multiple(key, &items);
            }
        }

        // Last known: merge into existing
        for (source, prices) in [
            (Source::Auction, &auction_prices),
            (Source::Bazaar, &bazaar_prices),
        ] {
            if !prices.is_empty() {
                let items: Vec<(String, String)> = prices
                    .iter()
                    .map(|(k, v)| (k.clone(), v.to_string()))
                    .collect();
                p.hset_multiple(last_key(source), &items);
            }
        }

        // History accumulation
        if crate::CONFIG.enable_history {
            for (source, prices) in [
                (Source::Auction, &auction_prices),
                (Source::Bazaar, &bazaar_prices),
            ] {
                let sk = acc_sums_key(source);
                let ck = acc_counts_key(source);
                for (item, price) in prices {
                    p.hincr(sk, item, *price as i64);
                    p.hincr(ck, item, 1i64);
                }
            }
        }

        p.set("meta:last_updated", now.to_string());
        let () = p.query_async(&mut conn).await?;

        Ok(())
    }

    async fn rollover_day(
        &self,
        conn: &mut ConnectionManager,
        current_day: u64,
    ) -> Result<()> {
        let mut p = pipe();
        p.atomic();

        for source in [Source::Auction, Source::Bazaar] {
            // Shift existing days: 5->6, 4->5, ..., 0->1
            for day in (0..DAY_SLOTS - 1).rev() {
                p.rename(day_sums_key(source, day), day_sums_key(source, day + 1))
                    .ignore();
                p.rename(
                    day_counts_key(source, day),
                    day_counts_key(source, day + 1),
                )
                .ignore();
            }
            // Move accumulator into day:0
            p.rename(acc_sums_key(source), day_sums_key(source, 0))
                .ignore();
            p.rename(acc_counts_key(source), day_counts_key(source, 0))
                .ignore();
        }

        p.set("meta:day_acc_start", current_day.to_string());
        let () = p.query_async(conn).await?;

        Ok(())
    }

    pub async fn get_prices(&self, source: Source) -> Result<HashMap<String, u64>> {
        let mut conn = self.conn.clone();
        let raw: HashMap<String, String> = conn.hgetall(prices_key(source)).await?;
        Ok(raw
            .into_iter()
            .filter_map(|(k, v)| v.parse().ok().map(|v| (k, v)))
            .collect())
    }

    pub async fn get_last_known(&self, source: Source) -> Result<HashMap<String, u64>> {
        let mut conn = self.conn.clone();
        let raw: HashMap<String, String> = conn.hgetall(last_key(source)).await?;
        Ok(raw
            .into_iter()
            .filter_map(|(k, v)| v.parse().ok().map(|v| (k, v)))
            .collect())
    }

    pub async fn get_averages(&self, source: Source, days: u8) -> Result<HashMap<String, u64>> {
        let mut conn = self.conn.clone();
        let mut total_sums: HashMap<String, u64> = HashMap::new();
        let mut total_counts: HashMap<String, u32> = HashMap::new();

        // Current accumulator
        let sums: HashMap<String, String> = conn.hgetall(acc_sums_key(source)).await?;
        let counts: HashMap<String, String> = conn.hgetall(acc_counts_key(source)).await?;

        for (key, val) in sums {
            if let Ok(v) = val.parse::<u64>() {
                *total_sums.entry(key).or_insert(0) += v;
            }
        }
        for (key, val) in counts {
            if let Ok(v) = val.parse::<u32>() {
                *total_counts.entry(key).or_insert(0) += v;
            }
        }

        // Historical day slots
        let slots_to_read = (days as usize).saturating_sub(1).min(DAY_SLOTS);
        for day in 0..slots_to_read {
            let sums: HashMap<String, String> =
                conn.hgetall(day_sums_key(source, day)).await?;
            let counts: HashMap<String, String> =
                conn.hgetall(day_counts_key(source, day)).await?;

            for (key, val) in sums {
                if let Ok(v) = val.parse::<u64>() {
                    *total_sums.entry(key).or_insert(0) += v;
                }
            }
            for (key, val) in counts {
                if let Ok(v) = val.parse::<u32>() {
                    *total_counts.entry(key).or_insert(0) += v;
                }
            }
        }

        let mut averages = HashMap::new();
        for (key, sum) in total_sums {
            if let Some(&count) = total_counts.get(&key)
                && count > 0
            {
                averages.insert(key, sum / count as u64);
            }
        }
        Ok(averages)
    }

    pub async fn get_last_updated(&self) -> Result<u64> {
        let mut conn = self.conn.clone();
        let val: Option<String> = conn.get("meta:last_updated").await?;
        Ok(val.and_then(|s| s.parse().ok()).unwrap_or(0))
    }
}
