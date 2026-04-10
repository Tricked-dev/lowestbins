# Redis Store & Query Parameter Filtering — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace in-memory state with a Store abstraction (Redis + memory fallback), add `?type` and `?price` query parameters, separate auction/bazaar data.

**Architecture:** Enum-dispatch `Store` with `RedisStore` and `MemoryStore` variants. Fetch module produces separate auction/bazaar HashMaps. Server parses query params and reads from Store. Fire-and-forget writes keep fetch cycle fast.

**Tech Stack:** Rust, redis crate (tokio-comp + connection-manager), hyper, tokio, parking_lot

---

## File Map

**Create:**
- `src/store/mod.rs` — Store enum, Source enum, PriceQuery/SourceFilter types, dispatch methods
- `src/store/memory.rs` — MemoryStore implementation
- `src/store/redis.rs` — RedisStore implementation

**Modify:**
- `Cargo.toml` — Add `redis`, remove `dashmap`
- `src/lib.rs` — Remove AUCTIONS/LAST_UPDATED/DYNAMIC_CACHE statics, add STORE static, add REDIS_URL to Conf
- `src/main.rs` — Remove save-to-disk task, remove history persist task, simplify shutdown
- `src/server.rs` — Parse query params, read from Store, support ?type and ?price on all endpoints
- `src/fetch/mod.rs` — Return separate auction/bazaar HashMaps, call store.write_cycle via tokio::spawn
- `src/fetch/auctions.rs` — Return HashMap instead of writing to DashMap
- `src/fetch/bazaar.rs` — Return HashMap instead of writing to DashMap
- `src/history.rs` — Gut entirely, re-export nothing (module kept empty for now or removed)
- `flake.nix` — Add valkey to devShell

**Remove:**
- `dashmap` usage everywhere
- `history.rs` internals (PriceHistory, PersistData, HISTORY static, DYNAMIC_CACHE, persist_now, spawn_persist_task)
- SAVE_TO_DISK config and background save task
- AUCTIONS BTreeMap and auctions.json loading
- get_prices_map() usage from lib.rs

---

### Task 1: Dependencies and Store Types

**Files:**
- Modify: `Cargo.toml`
- Create: `src/store/mod.rs`

- [ ] **Step 1: Update Cargo.toml**

Add `redis` dependency, remove `dashmap`:

```toml
# Add to [dependencies]:
redis = { version = "0.27", features = ["tokio-comp", "connection-manager"] }

# Remove from [dependencies]:
# dashmap = { version = "6.1", features = ["serde"] }
```

- [ ] **Step 2: Create src/store/mod.rs with types and dispatch**

```rust
pub mod memory;
pub mod redis_store;

use std::collections::HashMap;
use crate::error::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Auction,
    Bazaar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriceMode {
    Available,
    Historical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFilter {
    Auction,
    Bazaar,
    All,
}

impl Default for PriceMode {
    fn default() -> Self { PriceMode::Historical }
}

impl Default for SourceFilter {
    fn default() -> Self { SourceFilter::All }
}

pub use memory::MemoryStore;
pub use redis_store::RedisStore;

pub enum Store {
    Redis(RedisStore),
    Memory(MemoryStore),
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

    /// Resolve prices for a given source filter and price mode.
    /// Handles the ?type=all merge (lowest wins) at this layer.
    pub async fn resolve_prices(&self, filter: SourceFilter, mode: PriceMode) -> Result<HashMap<String, u64>> {
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
                let (auction, bazaar) = tokio::join!(
                    get(Source::Auction),
                    get(Source::Bazaar),
                );
                let mut merged = auction?;
                for (key, price) in bazaar? {
                    let entry = merged.entry(key).or_insert(u64::MAX);
                    if price < *entry { *entry = price; }
                }
                Ok(merged)
            }
        }
    }

    /// Resolve averages for a given source filter.
    pub async fn resolve_averages(&self, filter: SourceFilter, days: u8) -> Result<HashMap<String, u64>> {
        match filter {
            SourceFilter::Auction => self.get_averages(Source::Auction, days).await,
            SourceFilter::Bazaar => self.get_averages(Source::Bazaar, days).await,
            SourceFilter::All => {
                // For ?type=all averages, we need raw sums/counts to do weighted average.
                // For simplicity, merge the per-source averages (this is average-of-available,
                // not weighted — acceptable given both sources sample at same rate).
                let (auction, bazaar) = tokio::join!(
                    self.get_averages(Source::Auction, days),
                    self.get_averages(Source::Bazaar, days),
                );
                let mut merged = auction?;
                for (key, price) in bazaar? {
                    let entry = merged.entry(key).or_insert(u64::MAX);
                    if price < *entry { *entry = price; }
                }
                Ok(merged)
            }
        }
    }
}

/// Parse query parameters from a URI query string.
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
```

- [ ] **Step 3: Register store module in lib.rs**

Add `pub mod store;` to src/lib.rs.

---

### Task 2: MemoryStore

**Files:**
- Create: `src/store/memory.rs`

- [ ] **Step 1: Implement MemoryStore**

```rust
use std::collections::{HashMap, VecDeque};
use parking_lot::RwLock;
use crate::error::Result;
use super::Source;

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

pub struct MemoryStore {
    auction_prices: RwLock<HashMap<String, u64>>,
    bazaar_prices: RwLock<HashMap<String, u64>>,
    auction_last: RwLock<HashMap<String, u64>>,
    bazaar_last: RwLock<HashMap<String, u64>>,
    history_auction_acc: RwLock<(HashMap<String, u64>, HashMap<String, u32>)>,
    history_bazaar_acc: RwLock<(HashMap<String, u64>, HashMap<String, u32>)>,
    history_auction_days: RwLock<VecDeque<(HashMap<String, u64>, HashMap<String, u32>)>>,
    history_bazaar_days: RwLock<VecDeque<(HashMap<String, u64>, HashMap<String, u32>)>>,
    last_updated: RwLock<u64>,
    day_acc_start: RwLock<u64>,
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

    fn acc_for(&self, source: Source) -> &RwLock<(HashMap<String, u64>, HashMap<String, u32>)> {
        match source {
            Source::Auction => &self.history_auction_acc,
            Source::Bazaar => &self.history_bazaar_acc,
        }
    }

    fn days_for(&self, source: Source) -> &RwLock<VecDeque<(HashMap<String, u64>, HashMap<String, u32>)>> {
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

        // Current prices (wipe and replace)
        *self.auction_prices.write() = auction_prices.clone();
        *self.bazaar_prices.write() = bazaar_prices.clone();

        // Last known (merge)
        self.auction_last.write().extend(auction_prices.iter().map(|(k, v)| (k.clone(), *v)));
        self.bazaar_last.write().extend(bazaar_prices.iter().map(|(k, v)| (k.clone(), *v)));

        // History
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

        // Historical day slots (up to days-1 completed days)
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
            if let Some(&count) = total_counts.get(&key) {
                if count > 0 {
                    averages.insert(key, sum / count as u64);
                }
            }
        }
        Ok(averages)
    }

    pub async fn get_last_updated(&self) -> Result<u64> {
        Ok(*self.last_updated.read())
    }
}
```

---

### Task 3: RedisStore

**Files:**
- Create: `src/store/redis_store.rs`

- [ ] **Step 1: Implement RedisStore**

```rust
use std::collections::HashMap;
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, pipe};
use crate::error::Result;
use super::Source;

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
                let _: () = conn.set("meta:day_acc_start", current_day.to_string()).await?;
            }
        }

        // Build the main pipeline
        let mut p = pipe();
        p.atomic();

        // Current prices: wipe and rewrite
        for (source, prices) in [(Source::Auction, &auction_prices), (Source::Bazaar, &bazaar_prices)] {
            let key = prices_key(source);
            p.del(key);
            if !prices.is_empty() {
                let items: Vec<(String, String)> = prices.iter()
                    .map(|(k, v)| (k.clone(), v.to_string()))
                    .collect();
                p.hset_multiple(key, &items);
            }
        }

        // Last known: merge into existing
        for (source, prices) in [(Source::Auction, &auction_prices), (Source::Bazaar, &bazaar_prices)] {
            if !prices.is_empty() {
                let items: Vec<(String, String)> = prices.iter()
                    .map(|(k, v)| (k.clone(), v.to_string()))
                    .collect();
                p.hset_multiple(last_key(source), &items);
            }
        }

        // History accumulation
        if crate::CONFIG.enable_history {
            for (source, prices) in [(Source::Auction, &auction_prices), (Source::Bazaar, &bazaar_prices)] {
                let sk = acc_sums_key(source);
                let ck = acc_counts_key(source);
                for (item, price) in prices {
                    p.hincr(sk, item, *price as i64);
                    p.hincr(ck, item, 1i64);
                }
            }
        }

        p.set("meta:last_updated", now.to_string());
        p.query_async(&mut conn).await?;

        Ok(())
    }

    async fn rollover_day(&self, conn: &mut ConnectionManager, current_day: u64) -> Result<()> {
        let mut p = pipe();
        p.atomic();

        for source in [Source::Auction, Source::Bazaar] {
            // Shift existing days: 5->6, 4->5, ..., 0->1
            for day in (0..DAY_SLOTS - 1).rev() {
                p.rename(day_sums_key(source, day), day_sums_key(source, day + 1)).ignore();
                p.rename(day_counts_key(source, day), day_counts_key(source, day + 1)).ignore();
            }
            // Move accumulator into day:0
            p.rename(acc_sums_key(source), day_sums_key(source, 0)).ignore();
            p.rename(acc_counts_key(source), day_counts_key(source, 0)).ignore();
        }

        p.set("meta:day_acc_start", current_day.to_string());
        p.query_async(conn).await?;

        Ok(())
    }

    pub async fn get_prices(&self, source: Source) -> Result<HashMap<String, u64>> {
        let mut conn = self.conn.clone();
        let raw: HashMap<String, String> = conn.hgetall(prices_key(source)).await?;
        Ok(raw.into_iter().filter_map(|(k, v)| v.parse().ok().map(|v| (k, v))).collect())
    }

    pub async fn get_last_known(&self, source: Source) -> Result<HashMap<String, u64>> {
        let mut conn = self.conn.clone();
        let raw: HashMap<String, String> = conn.hgetall(last_key(source)).await?;
        Ok(raw.into_iter().filter_map(|(k, v)| v.parse().ok().map(|v| (k, v))).collect())
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
            let sums: HashMap<String, String> = conn.hgetall(day_sums_key(source, day)).await?;
            let counts: HashMap<String, String> = conn.hgetall(day_counts_key(source, day)).await?;

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
            if let Some(&count) = total_counts.get(&key) {
                if count > 0 {
                    averages.insert(key, sum / count as u64);
                }
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
```

---

### Task 4: Refactor Fetch Module

**Files:**
- Modify: `src/fetch/mod.rs`
- Modify: `src/fetch/auctions.rs`
- Modify: `src/fetch/bazaar.rs`

- [ ] **Step 1: Refactor auctions.rs to return HashMap**

Change `parse_auctions` to return `HashMap<String, u64>` instead of writing to DashMap. Change `get_auctions` to return a HashMap per page. Remove all DashMap usage.

- [ ] **Step 2: Refactor bazaar.rs to return HashMap**

Change `get_bazaar_products` to return `HashMap<String, u64>` instead of writing to DashMap.

- [ ] **Step 3: Refactor fetch/mod.rs**

Collect page results as Vec<HashMap>, merge sequentially (keeping lowest), keep bazaar separate. Call `store.write_cycle(auction_prices, bazaar_prices)` via tokio::spawn. Remove DashMap import.

---

### Task 5: Refactor lib.rs

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Remove old statics, add Store**

Remove: `AUCTIONS`, `LAST_UPDATED`, `set_last_updates()`, `calc_next_update()`, `get_prices_map()` include, auctions.json loading.

Add: `STORE` static (Lazy<Store>), `REDIS_URL` env var to Conf.

Keep: `CONFIG`, `HTTP_CLIENT`, `API_URL`, `round_to_nearest_15`, `SOURCE`, `SPONSOR`, `UA`.

Update `calc_next_update` to work with the store's timestamp (or remove it if no longer needed — the server can compute cache headers differently).

---

### Task 6: Refactor server.rs

**Files:**
- Modify: `src/server.rs`

- [ ] **Step 1: Update all endpoint handlers**

Parse query params on each request. Use `STORE.resolve_prices(filter, mode)` for price endpoints. Use `STORE.resolve_averages(filter, days)` for history endpoints. The response function becomes async and calls Store methods.

---

### Task 7: Refactor main.rs

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Remove save-to-disk task, history persist task, simplify shutdown**

Remove the `if CONFIG.save_to_disk` block. Remove `history::spawn_persist_task()` and `history::persist_now()` calls. Simplify shutdown to just log and exit. The fetch loop just calls `fetch_auctions().await` which internally spawns the Store write.

---

### Task 8: Gut history.rs

**Files:**
- Modify: `src/history.rs`

- [ ] **Step 1: Remove all internals**

History is now handled by the Store. Remove PriceHistory, PersistData, HISTORY static, DYNAMIC_CACHE, AverageView, get_cache, update_history, persist_now, spawn_persist_task. The file can either be deleted or left empty.

---

### Task 9: Update flake.nix

**Files:**
- Modify: `flake.nix`

- [ ] **Step 1: Add valkey to devShell, remove SAVE_TO_DISK**

Add `valkey` to buildInputs. Remove `SAVE_TO_DISK=1` from shellHook. Add `REDIS_URL` comment in shellHook.
