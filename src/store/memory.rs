use std::collections::{HashMap, VecDeque};

use parking_lot::RwLock;

use super::Source;
use crate::error::Result;

const DAY_SLOTS: usize = 7;

type Accumulator = (HashMap<String, u64>, HashMap<String, u32>);
type DaySlots = VecDeque<Accumulator>;

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn day_start_of(ts: u64) -> u64 {
    ts - (ts % 86400)
}

pub struct MemoryStore {
    auction_prices: RwLock<HashMap<String, u64>>,
    bazaar_prices: RwLock<HashMap<String, u64>>,
    auction_last: RwLock<HashMap<String, u64>>,
    bazaar_last: RwLock<HashMap<String, u64>>,
    history_auction_acc: RwLock<Accumulator>,
    history_bazaar_acc: RwLock<Accumulator>,
    history_auction_days: RwLock<DaySlots>,
    history_bazaar_days: RwLock<DaySlots>,
    last_updated: RwLock<u64>,
    day_acc_start: RwLock<u64>,
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            auction_prices: RwLock::new(HashMap::new()),
            bazaar_prices: RwLock::new(HashMap::new()),
            auction_last: RwLock::new(HashMap::new()),
            bazaar_last: RwLock::new(HashMap::new()),
            history_auction_acc: RwLock::new((HashMap::new(), HashMap::new())),
            history_bazaar_acc: RwLock::new((HashMap::new(), HashMap::new())),
            history_auction_days: RwLock::new(VecDeque::new()),
            history_bazaar_days: RwLock::new(VecDeque::new()),
            last_updated: RwLock::new(0),
            day_acc_start: RwLock::new(day_start_of(now_secs())),
        }
    }

    fn acc_for(&self, source: Source) -> &RwLock<Accumulator> {
        match source {
            Source::Auction => &self.history_auction_acc,
            Source::Bazaar => &self.history_bazaar_acc,
        }
    }

    fn days_for(&self, source: Source) -> &RwLock<DaySlots> {
        match source {
            Source::Auction => &self.history_auction_days,
            Source::Bazaar => &self.history_bazaar_days,
        }
    }

    pub async fn write_cycle(
        &self,
        auction_prices: HashMap<String, u64>,
        bazaar_prices: HashMap<String, u64>,
    ) -> Result<()> {
        let now = now_secs();

        *self.auction_prices.write() = auction_prices.clone();
        *self.bazaar_prices.write() = bazaar_prices.clone();

        self.auction_last
            .write()
            .extend(auction_prices.iter().map(|(k, v)| (k.clone(), *v)));
        self.bazaar_last
            .write()
            .extend(bazaar_prices.iter().map(|(k, v)| (k.clone(), *v)));

        if crate::CONFIG.enable_history {
            let current_day = day_start_of(now);
            let mut day_start = self.day_acc_start.write();

            if current_day != *day_start {
                for source in [Source::Auction, Source::Bazaar] {
                    let acc = std::mem::take(&mut *self.acc_for(source).write());
                    let mut days = self.days_for(source).write();
                    days.push_front(acc);
                    if days.len() > DAY_SLOTS {
                        days.pop_back();
                    }
                }
                *day_start = current_day;
            }

            self.accumulate(Source::Auction, &auction_prices);
            self.accumulate(Source::Bazaar, &bazaar_prices);
        }

        *self.last_updated.write() = now;
        Ok(())
    }

    fn accumulate(&self, source: Source, prices: &HashMap<String, u64>) {
        let mut acc = self.acc_for(source).write();
        for (key, price) in prices {
            *acc.0.entry(key.clone()).or_insert(0) += price;
            *acc.1.entry(key.clone()).or_insert(0) += 1;
        }
    }

    pub async fn get_prices(&self, source: Source) -> Result<HashMap<String, u64>> {
        Ok(match source {
            Source::Auction => self.auction_prices.read().clone(),
            Source::Bazaar => self.bazaar_prices.read().clone(),
        })
    }

    pub async fn get_last_known(&self, source: Source) -> Result<HashMap<String, u64>> {
        Ok(match source {
            Source::Auction => self.auction_last.read().clone(),
            Source::Bazaar => self.bazaar_last.read().clone(),
        })
    }

    pub async fn get_averages(&self, source: Source, days: u8) -> Result<HashMap<String, u64>> {
        let mut total_sums: HashMap<String, u64> = HashMap::new();
        let mut total_counts: HashMap<String, u32> = HashMap::new();

        // Current accumulator
        {
            let acc = self.acc_for(source).read();
            for (key, sum) in &acc.0 {
                *total_sums.entry(key.clone()).or_insert(0) += sum;
            }
            for (key, count) in &acc.1 {
                *total_counts.entry(key.clone()).or_insert(0) += count;
            }
        }

        // Historical day slots
        {
            let day_slots = self.days_for(source).read();
            let slots_to_read = (days as usize).saturating_sub(1).min(day_slots.len());
            for (sums, counts) in day_slots.iter().take(slots_to_read) {
                for (key, sum) in sums {
                    *total_sums.entry(key.clone()).or_insert(0) += sum;
                }
                for (key, count) in counts {
                    *total_counts.entry(key.clone()).or_insert(0) += count;
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
        Ok(*self.last_updated.read())
    }
}
