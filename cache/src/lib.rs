/*
 * 上应小风筝  便利校园，一步到位
 * Copyright (C) 2020-2023 上海应用技术大学 上应小风筝团队
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use anyhow::Context;
use chrono::{Duration, Local};
use once_cell::sync::OnceCell;

pub const SCOPE_PUBLIC: u8 = 0;
pub const SCOPE_BALANCE: u8 = 1;

// TODO: Consider OS compatability
const SLED_CACHE_PATH: &'static str = "./.runtime-cache/";

static CACHE: OnceCell<SledCache> = OnceCell::new();

trait CacheItemOperation<T> {
    fn is_expired(&self) -> bool;
}

pub trait CacheOperation<T>
where
    T: bincode::Encode + bincode::Decode,
{
    fn get(&self, key: &[u8], timeout: Duration) -> anyhow::Result<Option<T>>;

    fn set(&self, key: &[u8], value: T) -> anyhow::Result<()>;

    fn flush(&self) -> anyhow::Result<()>;
}

#[derive(Debug)]
pub struct SledCache(sled::Db);

#[derive(Debug, bincode::Decode, bincode::Encode)]
struct CacheItem<T: bincode::Encode + bincode::Decode> {
    /// Unix timestamp
    pub last_update: i64,
    /// Value
    pub value: T,
}

#[macro_export]
macro_rules! this_type {
    ($v: expr) => {{
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let v = $v;
        type_name_of(v)
    }};
}

#[macro_export]
macro_rules! this_function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}

/// BKDR String hash algorithm
///
/// https://blog.csdn.net/djinglan/article/details/8812934
pub fn bkdr_hash(initial_value: u64, s: &[u8]) -> u64 {
    let mut r = initial_value;
    for ch in s {
        let t = r.overflowing_mul(131).0;
        r = t.overflowing_add(*ch as u64).0;
    }
    r
}

pub fn u64_to_u8_array(mut n: u64) -> [u8; 8] {
    let mut result = [0; 8];
    for byte in 0..8 {
        result[byte] = (n & 0xff) as u8;
        n >>= 8;
    }
    result
}

#[macro_export]
macro_rules! cache_calc_key {
    ($($arg: expr),*) => {{
        $crate::cache_calc_key!(scope = $crare::SCOPE_PUBLIC, $($arg),*)
    }};
    (scope = $scope: expr; $($arg: expr),*) => {{
        let scope = $scope;
        assert!($crate::this_type!(scope) == "u8");

        let func: &str = $crate::this_function!();
        let mut hash_key: u64 = $crate::bkdr_hash(0, func.as_bytes());
        // TODO: Improve performance
        $(
            let parameter: String = format!("{:?}", $arg);
            hash_key = $crate::bkdr_hash(hash_key, parameter.as_bytes());
        )*
        // Return cache key
        let mut result = [0u8; 9];
        result[0] = scope;
        result[1..].copy_from_slice(&$crate::u64_to_u8_array(hash_key));
        result
    }}
}

#[macro_export]
macro_rules! cache_query {
    (key = $($arg: expr),*; timeout = $timeout: expr) => {{
        $crate::cache_query!(key = $($arg),*; scope = $crate::SCOPE_PUBLIC; timeout = $timeout)
    }};
    (key = $($arg: expr),*; scope = $scope: expr; timeout = $timeout: expr) => {{
        use $crate::CacheOperation;

        let cache = $crate::get();
        let cache_key = $crate::cache_calc_key!(scope = $scope; $($arg),*);

        cache.get(&cache_key, $timeout)
    }};
}

#[macro_export]
macro_rules! cache_save {
    (key = $($arg: expr),*; value = $value: expr) => {{
        $crate::cache_save!(scope = $crate::SCOPE_PUBLIC; key = $($arg),*; value = $value)
    }};
    (scope = $scope: expr; key = $($arg: expr),*; value = $value: expr) => {{
        use $crate::CacheOperation;

        let cache = $crate::get();
        let cache_key = $crate::cache_calc_key!(scope = $scope; $($arg),*);
        if let Err(e) = cache.set(&cache_key, $value) {
            tracing::warn!("failed to write data back to cache.");
        }
    }};
}

#[macro_export]
macro_rules! cache_erase {
    (key = $($arg: expr),*) => {
        $crate::cache_erase!(scope = $crate::SCOPE_PUBLIC; key = $($arg),*)
    };
    (scope = $scope: expr; key = $($arg: expr),*) => {{
        use $crate::CacheOperation;

        let cache = $crate::get();
        let cache_key = $crate::cache_calc_key!($($arg),*);
        if let Err(e) = cache.erase(&cache_key) {
            tracing::warn!("failed to erase item in cache (key: {:?}): {}", cache_key, e);
        }
    }};
}

impl SledCache {
    pub fn open(sled_path: &str) -> anyhow::Result<Self> {
        let db = sled::Config::new()
            .mode(sled::Mode::HighThroughput)
            .path(sled_path)
            .open()?;
        Ok(Self(db))
    }

    /// Peek timestamp field without deserializing the hold CacheItem.
    ///
    /// Ref: https://github.com/bincode-org/bincode/blob/trunk/docs/spec.md
    fn peek_timestamp(value: &sled::IVec) -> i64 {
        // Assume that the machine use little endian
        assert!(value.len() >= 8);

        let mut result = 0i64;
        let ts_binary: &[u8] = &value.subslice(0, 8);

        // Following lines are equal to:
        // result |= ts_binary[0];
        // result |= ts_binary[1] << 8;
        // result |= ts_binary[2] << 16;
        // ...
        for byte in 0..8 {
            result |= (ts_binary[byte] as i64) << ((byte << 3) as i64);
        }
        result
    }

    pub fn erase(&self, key: &[u8]) -> anyhow::Result<()> {
        self.0.remove(key)?;
        Ok(())
    }
}

impl<T> CacheOperation<T> for SledCache
where
    T: bincode::Encode + bincode::Decode,
{
    fn get(&self, key: &[u8], timeout: Duration) -> anyhow::Result<Option<T>> {
        let result = self.0.get(key);
        match result {
            Ok(Some(value)) => {
                let last_update = Self::peek_timestamp(&value);
                let now = Local::now().timestamp();

                // Cache hit
                if now - last_update < timeout.num_seconds() {
                    let config = bincode::config::legacy();
                    // Note: if cache key is conflict, the decode process will return an error.
                    // Caller should not mark the query failed.
                    bincode::decode_from_slice(&value, config)
                        .map(|(item, _): (CacheItem<T>, usize)| Some(item.value))
                        .map_err(Into::into)
                } else {
                    // Cache expired
                    // Remove the old and return none
                    self.0
                        .remove(key)
                        .map(|_| None)
                        .with_context(|| format!("Hit the expired item, failed to delete."))
                }
            }
            Ok(None) => Ok(None),         // Cache miss
            Err(e) => Err(Into::into(e)), // Operation failed.
        }
    }

    fn set(&self, key: &[u8], value: T) -> anyhow::Result<()> {
        let now = Local::now();
        let config = bincode::config::legacy();
        let item: CacheItem<T> = CacheItem {
            last_update: now.timestamp(),
            value,
        };

        let value = bincode::encode_to_vec(item, config).map_err(anyhow::Error::from)?;
        self.0
            .insert(key, value)
            .map(|_| ())
            .with_context(|| format!("Failed to write cache"))
    }

    fn flush(&self) -> anyhow::Result<()> {
        self.0.flush().map(|_| ()).map_err(Into::into)
    }
}

pub fn initialize() {
    tracing::debug!("Opening cache database...");

    let cache_handler = SledCache::open(SLED_CACHE_PATH)
        .with_context(|| format!("Cloud not open cache database: {}", SLED_CACHE_PATH))
        .expect("Failed to initialize cache module.");

    CACHE.set(cache_handler).unwrap();
}

pub fn get() -> &'static SledCache {
    CACHE.get().unwrap()
}
