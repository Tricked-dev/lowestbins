# Redis Store & Query Parameter Filtering

**Date:** 2026-04-10
**Status:** Approved

## Problem

The API currently merges auction and bazaar prices into a single in-memory `BTreeMap` with no way to distinguish sources. `SAVE_TO_DISK` persists stale prices across restarts without making this explicit to consumers. There is no way to request only currently-listed items vs last-known prices. All state is in-process, limiting durability and future scalability.

## Goals

1. Add `?type=auction|bazaar|all` parameter to filter by price source
2. Add `?price=available|historical` parameter to distinguish current listings from last-known prices
3. Replace in-memory state with a `Store` abstraction backed by either Redis (Valkey) or an in-memory fallback
4. Maintain full backwards compatibility — default behavior (`?type=all&price=historical`) matches current output

## Non-Goals

- Horizontal scaling / multi-instance coordination (future concern)
- Price-range queries or sorted set usage
- Changing response shapes (same `{"ITEM": price}` format)

---

## API Surface

Two new optional query parameters on existing endpoints. Defaults preserve current behavior.

| Parameter | Values | Default | Description |
|-----------|--------|---------|-------------|
| `type` | `auction`, `bazaar`, `all` | `all` | Filter by price source |
| `price` | `available`, `historical` | `historical` | `available` = only items in latest fetch cycle. `historical` = last known price, even if no longer listed |

### Endpoint Matrix

| Endpoint | `type` | `price` | Notes |
|----------|--------|---------|-------|
| `/lowestbins`, `/lowestbins.json`, `/auctions/lowestbins` | yes | yes | Full price map |
| `/lowestbins.txt` | yes | yes | Plain text variant |
| `/auction/{item}`, `/lowestbin/{item}` | yes | yes | Single item lookup |
| `/averages/{1-7}day` | yes | no | History is inherently historical; `price` param not applicable |
| `/metrics` | yes | yes | Prometheus output |
| `/` | — | — | Updated to document new parameters |

Default (`?type=all&price=historical`) is identical to current behavior. No breaking change.

### "Available" Definition

An item is "available" if and only if it appeared in the most recent Hypixel API fetch cycle. If it's absent from the latest fetch, it is immediately unavailable — no grace period, no TTL.

---

## Store Abstraction

### Enum Dispatch (No Dynamic Dispatch)

A `Store` enum with two variants, decided once at startup. No `dyn`, no `async_trait`, no heap-allocated futures.

```rust
pub enum Source {
    Auction,
    Bazaar,
}

pub enum Store {
    Redis(RedisStore),
    Memory(MemoryStore),
}
```

Each method on `Store` matches on the variant and delegates. Monomorphized at compile time — the only runtime cost is a single branch prediction per call.

**Startup decision:** If `REDIS_URL` is set, construct `RedisStore`. Otherwise, construct `MemoryStore`.

### Method Surface

```rust
impl Store {
    // Bulk write — called once per fetch cycle, spawned as fire-and-forget async task.
    // Writes current prices, updates last-known, and accumulates history in one pipeline.
    async fn write_cycle(
        &self,
        auction_prices: HashMap<String, u64>,
        bazaar_prices: HashMap<String, u64>,
    ) -> Result<()>;

    // Read — called per HTTP request
    async fn get_prices(&self, source: Source) -> Result<HashMap<String, u64>>;
    async fn get_last_known(&self, source: Source) -> Result<HashMap<String, u64>>;
    async fn get_averages(&self, source: Source, days: u8) -> Result<HashMap<String, u64>>;
    async fn get_last_updated(&self) -> Result<u64>;
}
```

`write_cycle` bundles all writes into one method so RedisStore can pipeline them in a single `MULTI/EXEC`. The fetch loop calls `tokio::spawn(store.write_cycle(auction, bazaar))` — non-blocking, doesn't delay the next fetch.

---

## Redis Key Schema (RedisStore)

```
# Current prices — wiped and rewritten each fetch cycle
auction:prices        → Hash { "JUJU_SHORTBOW": "12300000", ... }
bazaar:prices         → Hash { "ENCHANTED_BOOK-SHARPNESS-7": "480000", ... }

# Last known prices — never deleted, only overwritten with newer values
auction:last          → Hash { "JUJU_SHORTBOW": "12300000", ... }
bazaar:last           → Hash { "ENCHANTED_BOOK-SHARPNESS-7": "480000", ... }

# History — current day accumulators (reset on day rollover)
# Sums and counts stored in separate hashes so HINCRBY works natively
history:auction:acc:sums   → Hash { "ITEM": u64_sum, ... }
history:auction:acc:counts → Hash { "ITEM": u32_count, ... }
history:bazaar:acc:sums    → Hash { "ITEM": u64_sum, ... }
history:bazaar:acc:counts  → Hash { "ITEM": u32_count, ... }

# History — completed days (7-day sliding window, 0 = most recent)
history:auction:day:0:sums   → Hash { "ITEM": u64_sum, ... }
history:auction:day:0:counts → Hash { "ITEM": u32_count, ... }
...
history:auction:day:6:sums   → Hash { "ITEM": u64_sum, ... }
history:auction:day:6:counts → Hash { "ITEM": u32_count, ... }
history:bazaar:day:0:sums    → Hash { "ITEM": u64_sum, ... }
history:bazaar:day:0:counts  → Hash { "ITEM": u32_count, ... }
...
history:bazaar:day:6:sums    → Hash { "ITEM": u64_sum, ... }
history:bazaar:day:6:counts  → Hash { "ITEM": u32_count, ... }

# Metadata
meta:last_updated     → String (epoch seconds)
meta:day_acc_start    → String (epoch seconds)
```

### Fetch Cycle Write Pattern

**Non-blocking writes.** The fetch cycle (dominated by ~2.5s of Hypixel network I/O) must not be slowed by Redis. After fetching from Hypixel, the write to Redis is spawned as an async task (fire-and-forget). The fetch cycle returns immediately; the next cycle runs on its own interval regardless.

All writes in a single `MULTI/EXEC` pipeline (one network round-trip, ~20-50ms local / ~100-200ms remote for ~10k items):

1. `DEL auction:prices` then `HSET auction:prices field1 val1 field2 val2 ...`
2. `DEL bazaar:prices` then `HSET bazaar:prices ...`
3. `HSET auction:last ...` (merge into existing — keys accumulate, never removed)
4. `HSET bazaar:last ...`
5. If `ENABLE_HISTORY`: `HINCRBY` on `history:*:acc:sums` and `history:*:acc:counts` (separate integer hashes)
6. `SET meta:last_updated <epoch>`

No pre-computed merged keys. The `?type=all` merge is cheap enough on the read path (two pipelined `HGETALL` + Rust-side merge, ~1-3ms). Pre-computing would add write complexity for negligible read savings.

### Day Rollover

Detected by comparing `meta:day_acc_start` to current UTC date:

1. Shift day slots: for each source, `RENAME day:5:sums day:6:sums`, `RENAME day:5:counts day:6:counts`, etc. down to `day:0 → day:1`
2. `RENAME history:auction:acc:sums history:auction:day:0:sums` (and `:counts`)
3. `RENAME history:bazaar:acc:sums history:bazaar:day:0:sums` (and `:counts`)
4. `DEL history:*:day:6:sums history:*:day:6:counts` (evict oldest day if >7)
5. Reset `meta:day_acc_start` to current day

### Query Resolution

| Parameters | Redis Commands |
|------------|----------------|
| `?type=auction&price=available` | `HGETALL auction:prices` |
| `?type=bazaar&price=available` | `HGETALL bazaar:prices` |
| `?type=all&price=available` | `HGETALL auction:prices` + `HGETALL bazaar:prices`, merge (lowest wins) |
| `?type=auction&price=historical` | `HGETALL auction:last` |
| `?type=bazaar&price=historical` | `HGETALL bazaar:last` |
| `?type=all&price=historical` | `HGETALL auction:last` + `HGETALL bazaar:last`, merge (lowest wins) |
| `/averages/3day?type=auction` | `HGETALL history:auction:day:{0,1,2}:sums` + `:counts`, compute per-item `total_sum / total_count` |
| `/averages/3day?type=all` | Read both sources' day slots, combine sums and counts per item, then `combined_sum / combined_count` (weighted average, not average-of-averages) |

All multi-key reads are pipelined (single network round-trip).

---

## MemoryStore

In-memory implementation using the same logical structure:

```rust
pub struct MemoryStore {
    auction_prices: RwLock<HashMap<String, u64>>,
    bazaar_prices: RwLock<HashMap<String, u64>>,
    auction_last: RwLock<HashMap<String, u64>>,
    bazaar_last: RwLock<HashMap<String, u64>>,
    // History accumulators mirror Redis split: separate sums and counts
    history_auction_acc: RwLock<(HashMap<String, u64>, HashMap<String, u32>)>,
    history_bazaar_acc: RwLock<(HashMap<String, u64>, HashMap<String, u32>)>,
    history_auction_days: RwLock<VecDeque<(HashMap<String, u64>, HashMap<String, u32>)>>,
    history_bazaar_days: RwLock<VecDeque<(HashMap<String, u64>, HashMap<String, u32>)>>,
    last_updated: RwLock<u64>,
    day_acc_start: RwLock<u64>,
}
```

Behaves identically to `RedisStore` but all in-process. No persistence across restarts. Used when `REDIS_URL` is not set.

### `?type=all` Handling

The `Source` enum has two variants: `Auction` and `Bazaar`. The `all` case is handled at the HTTP handler layer, not the store layer — the handler calls the store twice (once per source) and merges results. For prices: lowest wins. For averages: combine sums and counts per item, then divide (weighted average).

---

## What Gets Removed

| Component | Reason |
|-----------|--------|
| `AUCTIONS` static (`Mutex<BTreeMap<String, u64>>`) | Replaced by `Store` |
| `LAST_UPDATED` static (`Mutex<Instant>`) | Replaced by `Store::get_last_updated()` |
| `DYNAMIC_CACHE` static (`RwLock<HashMap<u8, Bytes>>`) | Averages computed from store on request |
| `SAVE_TO_DISK` env var + background save task | Redis handles persistence |
| `auctions.json` file I/O | No longer needed |
| `history.rs` `PriceHistory` struct + in-memory Vecs | History lives in store |
| `history.rs` `PersistData` / disk serialization | Redis handles persistence |
| `history.rs` `persist_now()` / graceful shutdown flush | No in-memory state to flush |
| Hardcoded prices from `build.rs` (`get_prices_map()`) | No local BTreeMap to seed |
| `dashmap` dependency | No in-memory concurrent map needed |

### What Stays

- **Fetch logic** (`fetch/auctions.rs`, `fetch/bazaar.rs`, `fetch/mod.rs`) — still pulls from Hypixel, but writes to `Store` instead of `DashMap`
- **NBT parsing, item ID normalization** — unchanged
- **HTTP server** (`server.rs`) — endpoints stay, handlers read from `Store` and parse query params
- **Webhook reporting** — unchanged
- **`CONFIG` / env var loading** — stays, with `REDIS_URL` added and `SAVE_TO_DISK` removed
- **`ENABLE_HISTORY`** — still gates whether history accumulators are written
- **Signal handling** — simplified (no flush needed)
- **`build.rs` display name generation** — still needed for `/metrics`

---

## Dependencies

### Add

- `redis` crate with `tokio-comp` + `connection-manager` features — async Valkey-compatible client with automatic reconnection and TLS support

### Remove

- `dashmap` — replaced by `Store` abstraction

---

## Development Setup

### Nix Flake

Add `valkey` to `devShell` packages so `valkey-server` is available via `nix develop`.

### Runtime Store Selection

No external setup required for development:

- If `REDIS_URL` is set → `RedisStore` (connects to Valkey/Redis at that URL)
- If `REDIS_URL` is not set → `MemoryStore` (in-process, no persistence)

Developers can just `cargo run` with no Valkey instance and get full functionality via `MemoryStore`. To test with Valkey: `nix develop`, start `valkey-server`, set `REDIS_URL=redis://localhost:6379`.

---

## Configuration

### Environment Variables

| Variable | Default | Change |
|----------|---------|--------|
| `REDIS_URL` | (none) | **New.** If set, use `RedisStore`. Supports `redis://`, `rediss://` (TLS) |
| `SAVE_TO_DISK` | — | **Removed.** |
| `ENABLE_HISTORY` | `"0"` | Unchanged. Gates history accumulation in both store backends |
| All others | — | Unchanged |
