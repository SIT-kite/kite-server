use anyhow::Context;
use once_cell::sync::OnceCell;

use crate::config;

// TODO: Consider OS compatability
const SLED_CACHE_PATH: &'static str = "./cache/";

static CACHE: OnceCell<SledCache> = OnceCell::new();

pub trait CacheOperation<T> {
    fn should_update(&self, key: &str) -> bool;

    fn get(&self, key: &str) -> anyhow::Result<Option<T>>;

    fn set(&self, key: &str, value: T);

    fn flush(&self);
}

#[derive(Debug)]
pub struct SledCache(sled::Db);

impl SledCache {
    pub fn open(sled_path: &str) -> anyhow::Result<Self> {
        let db = sled::Config::new()
            .mode(sled::Mode::HighThroughput)
            .path(sled_path)
            .open()?;
        Ok(Self(db))
    }
}

pub fn initialize() {
    let sled_path = config::get()
        .cache
        .clone()
        .unwrap_or_else(|| SLED_CACHE_PATH.to_string());
    let cache_handler = SledCache::open(&sled_path)
        .with_context(|| format!("Cloud not open cache database: {}", sled_path))
        .expect("Failed to initialize cache module.");

    CACHE.set(cache_handler).unwrap();
}

pub fn get() -> &'static SledCache {
    CACHE.get().unwrap()
}
