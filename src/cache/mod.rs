use crate::helpers::{SystemTimeProvider, UnixTimeProvider};
use rustc_hash::FxHasher;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash};
use std::sync::Arc;
use std::time::Duration;

type FxDashMap<K, V> = dashmap::DashMap<K, V, BuildHasherDefault<FxHasher>>;

const MAX_CACHE_SIZE: usize = 2_097_152;
const DROP_UNUSED_PERIOD_SECS: u64 = 60 * 60;

pub enum CacheItemPolicy {
    AbsoluteExpiration(Duration),
    NoExpiration,
}

struct CacheItem<V: Debug> {
    pub value: Arc<V>,
    pub policy: CacheItemPolicy,
    pub created: u64,
    pub last_used: u64,
}

impl<V: Debug> CacheItem<V> {
    fn new(value: V, policy: CacheItemPolicy, created: u64) -> Self {
        Self {
            value: Arc::new(value),
            policy,
            created,
            last_used: created,
        }
    }
}

pub type MemoryCache<K, V> = MemoryCacheBase<K, V, SystemTimeProvider>;

impl<K: Eq + Hash + Debug, V: Debug> MemoryCache<K, V> {
    pub fn new() -> Self {
        Self::with_clock(SystemTimeProvider)
    }
}

pub struct MemoryCacheBase<K: Eq + Hash + Debug, V: Debug, T: UnixTimeProvider> {
    clock: T,
    cache: FxDashMap<K, CacheItem<V>>,
}

impl<K: Eq + Hash + Debug, V: Debug, T: UnixTimeProvider> MemoryCacheBase<K, V, T> {
    pub fn with_clock(clock: T) -> Self {
        Self {
            clock,
            cache: FxDashMap::default(),
        }
    }

    pub fn add(&self, key: K, value: V, policy: CacheItemPolicy) {
        self.drop_expired();

        if self.cache.len() >= MAX_CACHE_SIZE {
            self.drop_unused_for(DROP_UNUSED_PERIOD_SECS);
        }

        self.cache.insert(
            key,
            CacheItem::new(value, policy, self.clock.unix_time_as_secs()),
        );
    }

    pub fn get(&self, key: &K) -> Option<Arc<V>> {
        self.cache.remove_if(key, |_, item| !self.is_valid(item));
        self.cache.get_mut(key).map(|mut item| {
            item.last_used = self.clock.unix_time_as_secs();
            Arc::clone(&item.value)
        })
    }

    fn drop_expired(&self) {
        self.cache.retain(|_, item| self.is_valid(item));
    }

    fn is_valid(&self, item: &CacheItem<V>) -> bool {
        match item.policy {
            CacheItemPolicy::NoExpiration => true,
            CacheItemPolicy::AbsoluteExpiration(duration) => {
                let (created, duration, now) = (
                    item.created,
                    duration.as_secs(),
                    self.clock.unix_time_as_secs(),
                );
                created + duration > now
            }
        }
    }

    fn drop_unused_for(&self, period: u64) {
        let now = self.clock.unix_time_as_secs();
        self.cache.retain(|_, item| item.last_used + period > now);
    }
}
