use std::{
    collections::{HashMap, VecDeque},
    fs,
};

use hyper::body::Bytes;
use once_cell::sync::Lazy;
use parking_lot::{Mutex, RwLock};
use serde::{ser::SerializeMap, Deserialize, Serialize};
use tokio::task;

const DAY_SLOTS: usize = 7;
const PERSIST_PATH: &str = "history.json";

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn day_start_of(ts: u64) -> u64 {
    ts - (ts % 86400)
}

// Disk format uses HashMap<String, _> per slot so each slot is self-describing.
// This eliminates any risk of item_index / Vec length mismatch across restarts
// and makes new items appearing between restarts safe automatically.
// Runtime format stays as Vec<u64> with a shared index for memory efficiency.
#[derive(Serialize, Deserialize)]
struct PersistData {
    day_acc_start: u64,
    // (item_id -> (sum, count)) for the partial current day
    day_acc: HashMap<String, (u64, u32)>,
    // (day_start_ts, item_id -> (sum, count)) for completed days
    day_slots: Vec<(u64, HashMap<String, (u64, u32)>)>,
}

struct PriceHistory {
    item_index: HashMap<String, usize>,
    items: Vec<String>,

    // Running accumulator for the current calendar day
    day_acc_start: u64,
    day_acc_sum: Vec<u64>,
    day_acc_count: Vec<u32>,

    // Up to 7 completed calendar days
    // Each entry: (day_start_ts, price_sum_per_item, snapshot_count_per_item)
    day_slots: VecDeque<(u64, Vec<u64>, Vec<u32>)>,
}

impl PriceHistory {
    fn load_or_new() -> Self {
        if let Ok(bytes) = fs::read(PERSIST_PATH)
            && let Ok(p) = serde_json::from_slice::<PersistData>(&bytes) {
                return Self::from_persist(p);
            }
        Self {
            item_index: HashMap::new(),
            items: Vec::new(),
            day_acc_start: day_start_of(now_secs()),
            day_acc_sum: Vec::new(),
            day_acc_count: Vec::new(),
            day_slots: VecDeque::new(),
        }
    }

    // Reconstructs the array-indexed runtime format from the self-describing
    // HashMap disk format. Builds a unified item index from all slots so new
    // items that appeared between restarts integrate without any special casing.
    fn from_persist(p: PersistData) -> Self {
        let mut item_index: HashMap<String, usize> = HashMap::new();
        let mut items: Vec<String> = Vec::new();

        // Register all keys from every slot upfront so the index is complete
        // before we start converting slot data.
        let mut ensure = |key: &str| {
            if !item_index.contains_key(key) {
                let idx = items.len();
                item_index.insert(key.to_owned(), idx);
                items.push(key.to_owned());
            }
        };
        for key in p.day_acc.keys() {
            ensure(key);
        }
        for (_, slot) in &p.day_slots {
            for key in slot.keys() {
                ensure(key);
            }
        }

        let n = items.len();

        let mut day_acc_sum = vec![0u64; n];
        let mut day_acc_count = vec![0u32; n];
        for (key, (sum, count)) in &p.day_acc {
            if let Some(&idx) = item_index.get(key) {
                day_acc_sum[idx] = *sum;
                day_acc_count[idx] = *count;
            }
        }

        let day_slots: VecDeque<_> = p
            .day_slots
            .into_iter()
            .map(|(ts, map)| {
                let mut sum = vec![0u64; n];
                let mut count = vec![0u32; n];
                for (key, (s, c)) in map {
                    if let Some(&idx) = item_index.get(&key) {
                        sum[idx] = s;
                        count[idx] = c;
                    }
                }
                (ts, sum, count)
            })
            .collect();

        tracing::info!(
            "Loaded price history from disk ({} items, {} completed days)",
            n,
            day_slots.len()
        );

        Self {
            item_index,
            items,
            day_acc_start: p.day_acc_start,
            day_acc_sum,
            day_acc_count,
            day_slots,
        }
    }

    // Registers a new item, expanding all existing slot vecs with 0 to keep
    // indices aligned. The 0 values are excluded from averages via count guards.
    fn ensure_item(&mut self, key: &str) -> usize {
        if let Some(&idx) = self.item_index.get(key) {
            return idx;
        }
        let idx = self.items.len();
        self.item_index.insert(key.to_owned(), idx);
        self.items.push(key.to_owned());
        self.day_acc_sum.push(0);
        self.day_acc_count.push(0);
        for (_, sum, count) in &mut self.day_slots {
            sum.push(0);
            count.push(0);
        }
        idx
    }

    pub fn push_snapshot<I>(&mut self, prices: I)
    where
        I: IntoIterator<Item = (String, u64)>,
    {
        let ts = now_secs();
        let current_day = day_start_of(ts);

        // Day rollover: finalize the current accumulator into a completed day slot
        if current_day != self.day_acc_start {
            let n = self.items.len();
            let sum = std::mem::replace(&mut self.day_acc_sum, vec![0u64; n]);
            let count = std::mem::replace(&mut self.day_acc_count, vec![0u32; n]);
            self.day_slots.push_back((self.day_acc_start, sum, count));
            // Maintain sliding window by removing the oldest day slot
            if self.day_slots.len() > DAY_SLOTS {
                self.day_slots.pop_front();
            }
            self.day_acc_start = current_day;
        }

        for (key, price) in prices {
            let idx = self.ensure_item(&key);
            self.day_acc_sum[idx] += price;
            self.day_acc_count[idx] += 1;
        }
    }

    fn to_persist(&self) -> PersistData {
        let day_acc = self
            .items
            .iter()
            .enumerate()
            .filter(|(i, _)| self.day_acc_count[*i] > 0)
            .map(|(i, key)| (key.clone(), (self.day_acc_sum[i], self.day_acc_count[i])))
            .collect();

        let day_slots = self
            .day_slots
            .iter()
            .map(|(ts, sum, count)| {
                let map = self
                    .items
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| count[*i] > 0)
                    .map(|(i, key)| (key.clone(), (sum[i], count[i])))
                    .collect();
                (*ts, map)
            })
            .collect();

        PersistData {
            day_acc_start: self.day_acc_start,
            day_acc,
            day_slots,
        }
    }
}

// A zero-allocation view for serialization
struct AverageView<'a> {
    history: &'a PriceHistory,
    days: usize,
}

impl<'a> Serialize for AverageView<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let n = self.history.items.len();
        let mut sum = vec![0u64; n];
        let mut count = vec![0u32; n];

        // Process current partial day accumulator
        for i in 0..n.min(self.history.day_acc_sum.len()) {
            sum[i] += self.history.day_acc_sum[i];
            count[i] += self.history.day_acc_count[i];
        }

        // Add historical slots within window
        let cutoff_ts = self
            .history
            .day_acc_start
            .saturating_sub((self.days as u64).saturating_sub(1) * 86400);
        for (ts, d_sum, d_count) in self.history.day_slots.iter().rev() {
            // Break early if the historical slot falls outside the requested chronological window
            if *ts < cutoff_ts {
                break;
            }
            for i in 0..n.min(d_sum.len()) {
                sum[i] += d_sum[i];
                count[i] += d_count[i];
            }
        }

        // Count valid entries to pre-allocate map capacity in serialization
        let valid_count = count.iter().filter(|&&c| c > 0).count();
        let mut map = serializer.serialize_map(Some(valid_count))?;

        //  Stream calculations directly to the serializer
        for i in 0..n {
            if count[i] > 0 {
                map.serialize_entry(&self.history.items[i], &(sum[i] / count[i] as u64))?;
            }
        }

        map.end()
    }
}

static HISTORY: Lazy<Mutex<PriceHistory>> = Lazy::new(|| Mutex::new(PriceHistory::load_or_new()));
static DYNAMIC_CACHE: Lazy<RwLock<HashMap<u8, Bytes>>> = Lazy::new(|| RwLock::new(HashMap::with_capacity(DAY_SLOTS)));

pub fn get_cache(days: u8) -> Option<Bytes> {
    DYNAMIC_CACHE.read().get(&days).cloned()
}

/// Pushes a new price snapshot and recomputes all average caches.
/// Accept any iterator that yields (String, u64) to avoid unnecessary collection conversions.
pub fn update_history<I>(prices: I)
where
    I: IntoIterator<Item = (String, u64)> + Send + 'static,
{
    if !crate::CONFIG.enable_history {
        return;
    } // Feature guard

    tokio::spawn(async move {
        let result = task::spawn_blocking(move || {
            let mut h = HISTORY.lock();
            h.push_snapshot(prices);
            let mut new_caches = HashMap::with_capacity(DAY_SLOTS);
            for days in 1..=DAY_SLOTS {
                let view = AverageView { history: &h, days };
                let bytes = match serde_json::to_vec(&view) {
                    Ok(v) => Bytes::from(v),
                    Err(e) => {
                        tracing::error!("Failed to serialize {days}day average: {e}");
                        Bytes::from_static(b"{}")
                    }
                };
                new_caches.insert(days as u8, bytes);
            }
            new_caches
        })
        .await;

        match result {
            Ok(caches) => {
                *DYNAMIC_CACHE.write() = caches;
            }
            Err(e) => tracing::error!("History update panicked: {e}"),
        }
    });
}

/// Collects current memory state and writes it to disk immediately.
/// This is a blocking operation to ensure data is written before process exit.
pub fn persist_now() {
    let data = HISTORY.lock().to_persist();
    if data.day_acc.is_empty() && data.day_slots.is_empty() {
        tracing::warn!("Persistence skipped: History data is empty. Protecting existing disk data.");
        return;
    }
    // Atomic write
    match serde_json::to_vec(&data) {
        Ok(bytes) => {
            let temp_path = format!("{}.tmp", PERSIST_PATH);
            if let Err(e) = fs::write(&temp_path, &bytes) {
                tracing::error!("Failed to write temporary history file: {e}");
                return;
            }
            if let Err(e) = fs::rename(&temp_path, PERSIST_PATH) {
                tracing::error!("Failed to commit history file: {e}");
                let _ = fs::remove_file(&temp_path);
            } else {
                tracing::debug!("History successfully persisted to disk ({} bytes)", bytes.len());
            }
        }
        Err(e) => tracing::error!("Failed to serialize history data: {e}"),
    }
}

/// Spawns a background task that persists data every hour.
pub fn spawn_persist_task() {
    tokio::spawn(async {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600));
        loop {
            interval.tick().await;
            // Use spawn_blocking for periodic saves to avoid stalling the executor
            task::spawn_blocking(persist_now);
        }
    });
}
